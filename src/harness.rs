//! Agent 主循环（solution-map §3）：LLM 决策 → 工具调度 → 增量记账 → 收敛/熔断。
//! 收敛 = submit_report（结构化报告 + 证据校验）或 prose 降级；空转结构性禁止（D002）；
//! 上下文装不下 → 确定性压缩 + recall 钻取（D008）。

use crate::audit::Audit;
use crate::context;
use crate::errors::{CsError, CsResult, LLM_FUSE};
use crate::evidence::{EvidenceStore, info_keys};
use crate::llm::{ChatMessage, ChatRequest, LlmProvider, ToolCallSpec, ToolSchema};
use crate::report::{Evidence, Finding, Report, ReportStats};
use crate::tools::ToolRegistry;
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 熔断阈值：连续无进展步（重复调用 / 非法调用 / 零增量执行）。
pub const MAX_NO_PROGRESS_STREAK: u32 = 5;
/// 打转转向阈值：连续零信息增量的回合数。
pub const ZERO_GAIN_STEER_THRESHOLD: u32 = 2;
const SUBMIT_TOOL: &str = "submit_report";
const RECALL_TOOL: &str = "recall";

#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub report: Report,
    /// 人类渲染版报告（render_human）。
    pub answer: String,
    pub turns: u32,
    pub tool_calls: u32,
    pub audit_path: PathBuf,
}

pub struct Harness {
    provider: Arc<dyn LlmProvider>,
    tools: ToolRegistry,
    audit: Audit,
    evidence: Mutex<EvidenceStore>,
    model: String,
    context_tokens: u64,
    compact_percent: u64,
    first_user_suffix: Option<String>,
}

impl Harness {
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        tools: ToolRegistry,
        audit: Audit,
        model: String,
        context_tokens: u64,
        compact_percent: u64,
    ) -> Self {
        Self {
            provider,
            tools,
            audit,
            evidence: Mutex::new(EvidenceStore::default()),
            model,
            context_tokens,
            compact_percent,
            first_user_suffix: None,
        }
    }

    /// 首条用户消息后缀（P003 E2 v2：召回块 + 任务相关导航图，随首条注入一次，非 system——
    /// D005 分层：system 恒定，任务派生数据走 user 消息；「进门看一眼图就收起来」）。
    pub fn with_first_user_suffix(mut self, suffix: String) -> Self {
        self.first_user_suffix = Some(suffix);
        self
    }

    pub async fn run(&self, task: &str) -> CsResult<RunOutcome> {
        let mut messages = vec![
            ChatMessage::System {
                content: crate::prompt::SYSTEM_PROMPT.into(),
            },
            ChatMessage::User {
                content: task.to_string(),
            },
        ];
        if let Some(suffix) = &self.first_user_suffix
            && let Some(ChatMessage::User { content }) = messages.last_mut()
        {
            content.push_str("\n\n");
            content.push_str(suffix);
        }
        let mut seen_calls: HashSet<String> = HashSet::new();
        let mut seen_keys: HashSet<String> = HashSet::new();
        let mut no_progress: u32 = 0;
        let mut zero_gain_streak: u32 = 0;
        let mut prose_streak: u32 = 0;
        let mut turns: u32 = 0;
        let mut executed: u32 = 0;
        let mut total_tokens: u64 = 0;
        let mut llm_duration_ms: u64 = 0;

        loop {
            // D013：装不下才压缩（阈值 = 模型窗口 × 百分比）；三段式：持久化 → handoff → 压缩
            let threshold =
                context::compact_threshold_tokens(self.context_tokens, self.compact_percent);
            if context::should_compact(&messages, threshold) {
                // Phase 2 先行：承上启下 handoff（先于压缩生成，确定性零 LLM）；压缩是纯函数，无驱逐则静默
                // P005 R4.1：recall 起点必须是 1——审计 seq 从 1 起号，硬编码 2 会让首条记录永久失联
                let audit_to = self.audit.last_seq();
                let handoff = context::build_handoff(task, 1, audit_to);
                let (compacted, info) =
                    context::compact(std::mem::take(&mut messages), handoff, context::KEEP_RECENT);
                if info.evicted_count > 0 {
                    // Phase 3 提交（有真实驱逐才留痕，不产噪音）
                    self.audit.record(
                        "compaction_begin",
                        &serde_json::json!({"threshold_tokens": threshold}),
                    )?;
                    messages = compacted;
                    self.audit.record(
                        "compaction",
                        &serde_json::json!({"evicted_count": info.evicted_count}),
                    )?;
                } else {
                    messages = compacted; // no-op 原样（窗口尚无可驱逐内容）
                }
            }

            let req = ChatRequest {
                model: self.model.clone(),
                messages: messages.clone(),
                tools: {
                    let mut ts = self.tools.schemas();
                    ts.extend(self.builtin_schemas());
                    ts
                },
            };
            let started = Instant::now();
            let resp = match self.provider.chat(&req).await {
                Ok(r) => r,
                Err(e) => {
                    // 失败调用也留痕（D003.2 全留痕）；留痕本身失败也必须可见（P004 T3 吞错清零）
                    if let Err(audit_err) = self.audit.record(
                        "llm_error",
                        &serde_json::json!({"turn": turns, "code": e.code.to_string(), "message": e.message}),
                    ) {
                        tracing::warn!("llm_error 审计留痕失败: {audit_err}");
                    }
                    return Err(e);
                }
            };
            let duration_ms = started.elapsed().as_millis() as u64;
            llm_duration_ms += duration_ms;
            total_tokens += resp.usage.total_tokens as u64;
            turns += 1;
            self.audit.record(
                "llm",
                &serde_json::json!({
                    "turn": turns,
                    "model": self.model,
                    "duration_ms": duration_ms,
                    "prompt_tokens": resp.usage.prompt_tokens,
                    "completion_tokens": resp.usage.completion_tokens,
                    "total_tokens": resp.usage.total_tokens,
                    "text": resp.text.clone(),
                    "tool_call_count": resp.tool_calls.len(),
                }),
            )?;

            // 无工具调用：先引导 submit_report，再降级接受 prose（诚实标注 degraded）
            if resp.tool_calls.is_empty() {
                prose_streak += 1;
                if prose_streak <= 1 {
                    // P005 R6.1：模型首轮 prose 入史——只推引导会把模型刚说的话丢出上下文，
                    // 且连续两条 User 消息部分严格网关拒收
                    messages.push(ChatMessage::Assistant {
                        content: resp.text.clone(),
                        tool_calls: Vec::new(),
                    });
                    messages.push(ChatMessage::User {
                        content: "请调用 submit_report 工具提交结构化报告（answer/findings/dead_ends/confidence），evidence 引用本会话真实读过的 file:lines。".into(),
                    });
                    self.audit
                        .record("prose_steer", &serde_json::json!({"turn": turns}))?;
                    continue;
                }
                let answer = resp.text.unwrap_or_default();
                let stats = ReportStats {
                    turns,
                    tool_calls: executed,
                    duration_ms: llm_duration_ms,
                    total_tokens,
                };
                let report = Report::degraded_prose(task, &answer, stats);
                self.audit.record(
                    "answer",
                    &serde_json::json!({"answer": answer, "degraded": true}),
                )?;
                return Ok(RunOutcome {
                    answer: report.render_human(),
                    report,
                    turns,
                    tool_calls: executed,
                    audit_path: self.audit.path().to_path_buf(),
                });
            }

            messages.push(ChatMessage::Assistant {
                content: resp.text.clone(),
                tool_calls: resp.tool_calls.clone(),
            });
            // 本轮有真实工具调用 = 模型在工作，prose 计数归零（否则下一轮 prose 直接被降级，
            // 剥夺了引导 submit_report 的机会——P004 T2.2）
            prose_streak = 0;

            for call in &resp.tool_calls {
                // 0) submit_report：收敛通道（不走去重——被拒后允许修正重提）
                if call.name == SUBMIT_TOOL {
                    let Some(a) = parse_args(&call.arguments) else {
                        no_progress += 1;
                        if let Some(fuse) = fuse_if_hit(no_progress, &self.audit)? {
                            return Err(fuse);
                        }
                        self.reject_call(call, "submit_report 参数不是合法 JSON。", &mut messages)
                            .await?;
                        continue;
                    };
                    match self.build_report(
                        task,
                        &a,
                        turns,
                        executed,
                        llm_duration_ms,
                        total_tokens,
                    ) {
                        Ok(report) => {
                            self.audit.record(
                                "report",
                                &serde_json::json!({
                                    "answer": report.answer,
                                    "findings": report.findings.len(),
                                    "confidence": report.confidence,
                                    "degraded": false,
                                }),
                            )?;
                            let answer = report.render_human();
                            return Ok(RunOutcome {
                                report,
                                answer,
                                turns,
                                tool_calls: executed,
                                audit_path: self.audit.path().to_path_buf(),
                            });
                        }
                        Err(reject) => {
                            no_progress += 1;
                            if let Some(fuse) = fuse_if_hit(no_progress, &self.audit)? {
                                return Err(fuse);
                            }
                            self.reject_call(call, &format!("报告被拒：{reject}"), &mut messages)
                                .await?;
                            continue;
                        }
                    }
                }

                // 0.5) recall：审计原文钻取（D008；幂等元操作，不走去重）
                if call.name == RECALL_TOOL {
                    let Some(a) = parse_args(&call.arguments) else {
                        self.reject_call(call, "recall 参数不是合法 JSON。", &mut messages)
                            .await?;
                        continue;
                    };
                    let from = a.get("from").and_then(Value::as_u64).unwrap_or(0);
                    let to = a.get("to").and_then(Value::as_u64).unwrap_or(from);
                    let lines = self.audit.read_range(from, to)?;
                    executed += 1;
                    self.audit.record(
                        "recall",
                        &serde_json::json!({"from": from, "to": to, "lines": lines.len()}),
                    )?;
                    let msg = context::recall_message(&lines);
                    if let ChatMessage::User { content } = &msg {
                        let keys = info_keys(content);
                        if keys.iter().any(|k| seen_keys.insert(k.clone())) {
                            no_progress = 0;
                            zero_gain_streak = 0;
                        }
                    }
                    messages.push(msg);
                    continue;
                }

                // 1) 去重：同工具+同参（canonical）直接拒绝并强制转向（D002 空转结构性禁止）
                let key = canonical_call(&call.name, &call.arguments);
                if !seen_calls.insert(key) {
                    // 重复调用 = 无进展步，计入打转计数（否则坏模型可永远循环同一调用）
                    no_progress += 1;
                    if let Some(fuse) = fuse_if_hit(no_progress, &self.audit)? {
                        return Err(fuse);
                    }
                    self.reject_call(
                        call,
                        "重复调用已拒绝：同工具+同参数已执行过。请换工具/换角度，或用 submit_report 给出结论。",
                        &mut messages,
                    )
                    .await?;
                    continue;
                }

                // 2) 工具存在性 + 参数合法性
                let Some(args) = parse_args(&call.arguments) else {
                    no_progress += 1;
                    if let Some(fuse) = fuse_if_hit(no_progress, &self.audit)? {
                        return Err(fuse);
                    }
                    self.reject_call(
                        call,
                        "参数不是合法 JSON。请修正参数或换策略。",
                        &mut messages,
                    )
                    .await?;
                    continue;
                };
                let Some(tool) = self.tools.get(&call.name) else {
                    no_progress += 1;
                    if let Some(fuse) = fuse_if_hit(no_progress, &self.audit)? {
                        return Err(fuse);
                    }
                    self.reject_call(
                        call,
                        &format!(
                            "未知工具 {}。可用工具: {}（或用 submit_report 收敛）",
                            call.name,
                            tool_names(&self.tools)
                        ),
                        &mut messages,
                    )
                    .await?;
                    continue;
                };

                // 3) 执行 + 增量记账
                let seq = self.audit.record(
                    "tool_call",
                    &serde_json::json!({"turn": turns, "name": call.name, "args": args}),
                )?;
                // read 的精确路径要在 execute 消耗 args 之前摘出（FINDING-010 存证用）
                let read_exact_path = if call.name == "read" {
                    args.get("path").and_then(Value::as_str).map(str::to_string)
                } else {
                    None
                };
                match tool.execute(args).await {
                    Ok(output) => {
                        executed += 1;
                        self.audit.record(
                            "tool_result",
                            &serde_json::json!({"tool_call_seq": seq, "name": call.name, "output": output}),
                        )?;
                        self.evidence
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .observe(&output, seq);
                        if let Some(p) = &read_exact_path {
                            self.evidence
                                .lock()
                                .unwrap_or_else(|p| p.into_inner())
                                .observe_exact(p, seq);
                        }
                        let keys = info_keys(&output);
                        let gain = keys.iter().any(|k| seen_keys.insert(k.clone()));
                        if gain {
                            no_progress = 0;
                            zero_gain_streak = 0;
                        } else {
                            // 零增量执行也是无进展步：先计入，再触发转向（D002）
                            no_progress += 1;
                            zero_gain_streak += 1;
                        }
                        messages.push(ChatMessage::Tool {
                            call_id: call.id.clone(),
                            content: output,
                        });
                        // 打转转向：连续零增量达到阈值 → 注入转向指令（不是步数上限，D002）
                        if zero_gain_streak >= ZERO_GAIN_STEER_THRESHOLD {
                            zero_gain_streak = 0;
                            messages.push(ChatMessage::User {
                                content: "系统提示：连续多回合无新信息。请换工具/换检索角度，或调用 submit_report 给出结论。".into(),
                            });
                            if let Some(fuse) = fuse_if_hit(no_progress, &self.audit)? {
                                return Err(fuse);
                            }
                        }
                    }
                    Err(e) => {
                        // 工具执行错误也是无进展步（P004 T2.1）：否则坏模型可用千变参数无限触发
                        // 错误空转，永远不触熔断。错误回显给模型后照常计熔断。
                        no_progress += 1;
                        messages.push(ChatMessage::Tool {
                            call_id: call.id.clone(),
                            content: format!("工具错误: {e}"),
                        });
                        if let Some(fuse) = fuse_if_hit(no_progress, &self.audit)? {
                            return Err(fuse);
                        }
                    }
                }
            }
        }
    }

    /// submit_report → Report。证据校验：引用的 file 必须本会话真实观察到（否则拒绝并列出）。
    /// 内置工具 schema：submit_report（收敛）与 recall（钻取）由 harness 自己处理，但对 LLM 可见可调。
    fn builtin_schemas(&self) -> Vec<ToolSchema> {
        vec![
            ToolSchema {
                name: SUBMIT_TOOL.into(),
                description: "调查完成后提交结构化侦察报告（最终收敛动作）。evidence 必须引用本会话真实读过的文件。".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "required": ["answer", "findings", "confidence"],
                    "properties": {
                        "answer": {"type": "string", "description": "一段话结论"},
                        "findings": {"type": "array", "items": {"type": "object", "required": ["statement", "evidence"], "properties": {
                            "statement": {"type": "string"},
                            "evidence": {"type": "array", "items": {"type": "object", "required": ["file", "lines"], "properties": {
                                "file": {"type": "string"},
                                "lines": {"type": "string", "description": "如 7-22"}
                            }}}
                        }}},
                        "dead_ends": {"type": "array", "items": {"type": "string"}},
                        "confidence": {"type": "string", "enum": ["high", "medium", "low"]}
                    }
                }),
            },
            ToolSchema {
                name: RECALL_TOOL.into(),
                description: "按审计 seq 范围回读被压缩驱逐的原文（不重不漏）。args: {from, to}".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "required": ["from", "to"],
                    "properties": {
                        "from": {"type": "integer", "description": "起始 seq"},
                        "to": {"type": "integer", "description": "结束 seq"}
                    }
                }),
            },
        ]
    }

    fn build_report(
        &self,
        task: &str,
        args: &Value,
        turns: u32,
        executed: u32,
        duration_ms: u64,
        total_tokens: u64,
    ) -> Result<Report, String> {
        let answer = args
            .get("answer")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let confidence = args
            .get("confidence")
            .and_then(Value::as_str)
            .unwrap_or("medium")
            .to_lowercase();
        let mut findings: Vec<Finding> = Vec::new();
        let mut uncited: Vec<String> = Vec::new();
        if let Some(fs) = args.get("findings").and_then(Value::as_array) {
            for f in fs {
                let statement = f
                    .get("statement")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let mut evs: Vec<Evidence> = Vec::new();
                if let Some(evids) = f.get("evidence").and_then(Value::as_array) {
                    for e in evids {
                        let file = e
                            .get("file")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let lines = e
                            .get("lines")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        if file.is_empty() {
                            continue;
                        }
                        match self
                            .evidence
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .cite_seq(&file)
                        {
                            Some(seq) => evs.push(Evidence {
                                file,
                                lines,
                                audit_seq: Some(seq),
                            }),
                            None => uncited.push(format!("{file}:{lines}")),
                        }
                    }
                }
                if !statement.is_empty() {
                    findings.push(Finding {
                        statement,
                        evidence: evs,
                    });
                }
            }
        }
        let dead_ends: Vec<String> = args
            .get("dead_ends")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        if answer.trim().is_empty() {
            return Err("answer 为空".into());
        }
        if !uncited.is_empty() {
            return Err(format!(
                "evidence 引用了本会话未读过的文件: {}。请先 read 它们，或删除这些引用。",
                uncited.join(", ")
            ));
        }
        // FINDING-012：零 findings + 零证据 + 会话有真实读取 → 打回提炼（合法空手需 dead_ends 交代）
        let observed = self
            .evidence
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .observed_paths();
        if findings.is_empty() && !observed.is_empty() && dead_ends.is_empty() {
            let mut shown = observed.clone();
            shown.truncate(8);
            return Err(format!(
                "报告 findings 为 0，但本会话实际读取过 {} 个文件（如: {}）。\
                 请把读到的内容提炼为 findings 并逐条附上 evidence(file+lines) 后重新提交；\
                 若确实毫无收获，请把搜索过程写入 dead_ends 后再提交。",
                observed.len(),
                shown.join("、")
            ));
        }
        if !findings.is_empty() && !observed.is_empty() {
            let bare: Vec<usize> = findings
                .iter()
                .enumerate()
                .filter(|(_, f)| f.evidence.is_empty())
                .map(|(i, _)| i + 1)
                .collect();
            if !bare.is_empty() {
                let nums: Vec<String> = bare.iter().map(|n| n.to_string()).collect();
                return Err(format!(
                    "finding {} 未附任何证据：请从本会话已读文件中补上 evidence(file+lines)。",
                    nums.join("、")
                ));
            }
        }
        let report = Report {
            report_schema_version: crate::report::REPORT_SCHEMA_VERSION,
            task: task.to_string(),
            answer,
            findings,
            dead_ends,
            confidence,
            degraded: false,
            stats: ReportStats {
                turns,
                tool_calls: executed,
                duration_ms,
                total_tokens,
            },
        };
        report.validate().map_err(|e| e.to_string())?;
        Ok(report)
    }

    async fn reject_call(
        &self,
        call: &ToolCallSpec,
        reason: &str,
        messages: &mut Vec<ChatMessage>,
    ) -> CsResult<()> {
        self.audit.record(
            "tool_rejected",
            &serde_json::json!({"name": call.name, "arguments": call.arguments, "reason": reason}),
        )?;
        messages.push(ChatMessage::Tool {
            call_id: call.id.clone(),
            content: reason.to_string(),
        });
        Ok(())
    }
}

/// 连续无进展达到阈值 → 熔断（CS2099，故障域，非成本限制）。
fn fuse_if_hit(streak: u32, audit: &Audit) -> CsResult<Option<CsError>> {
    if streak >= MAX_NO_PROGRESS_STREAK {
        audit.record("fuse", &serde_json::json!({"no_progress_streak": streak}))?;
        // 熔断 = 会话级失败（P004 T4）：ERROR 级必打，不随 -v 静默
        tracing::error!("熔断触发：连续 {streak} 步无进展（CS2099）");
        Ok(Some(CsError::new(
            LLM_FUSE,
            format!("连续 {streak} 步无进展（重复/非法/零增量），熔断（故障域，非成本限制）"),
        )))
    } else {
        Ok(None)
    }
}

fn tool_names(reg: &ToolRegistry) -> String {
    let mut names: Vec<String> = reg.schemas().into_iter().map(|s| s.name).collect();
    names.push(SUBMIT_TOOL.into());
    names.join(", ")
}

/// 同参 canonical：name + 解析后再序列化的 JSON（键序归一）；解析失败退回原文。
fn canonical_call(name: &str, raw: &str) -> String {
    let args = serde_json::from_str::<Value>(raw)
        .map(|v| v.to_string())
        .unwrap_or_else(|_| raw.trim().to_string());
    format!("{name} {args}")
}

fn parse_args(raw: &str) -> Option<Value> {
    serde_json::from_str::<Value>(raw)
        .ok()
        .or(if raw.trim().is_empty() {
            Some(Value::Object(Default::default()))
        } else {
            None
        })
}

// 证据库与信息增量键已角色分离至 src/evidence.rs（P004 T6.2）。

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ChatResponse, Usage};
    use crate::tools::Tool;
    use std::collections::VecDeque;

    struct Scripted {
        script: Mutex<VecDeque<ChatResponse>>,
        seen: Arc<Mutex<Vec<ChatRequest>>>,
    }
    #[async_trait::async_trait]
    impl LlmProvider for Scripted {
        async fn chat(&self, req: &ChatRequest) -> CsResult<ChatResponse> {
            self.seen.lock().unwrap().push(req.clone());
            self.script
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| CsError::new(crate::errors::LLM_BAD_RESPONSE, "脚本耗尽"))
        }
    }

    struct Echo {
        calls: Arc<Mutex<u32>>,
        output: String,
    }
    #[async_trait::async_trait]
    impl Tool for Echo {
        fn name(&self) -> &'static str {
            "echo"
        }
        fn description(&self) -> String {
            "echo".into()
        }
        fn parameters(&self) -> Value {
            serde_json::json!({"type": "object"})
        }
        async fn execute(&self, _args: Value) -> CsResult<String> {
            *self.calls.lock().unwrap() += 1;
            Ok(self.output.clone())
        }
    }

    struct AlwaysErr;
    #[async_trait::async_trait]
    impl Tool for AlwaysErr {
        fn name(&self) -> &'static str {
            "always_err"
        }
        fn description(&self) -> String {
            "always fails".into()
        }
        fn parameters(&self) -> Value {
            serde_json::json!({"type": "object"})
        }
        async fn execute(&self, _args: Value) -> CsResult<String> {
            Err(CsError::new(crate::errors::USER_INPUT, "boom"))
        }
    }

    fn resp_text(s: &str) -> ChatResponse {
        ChatResponse {
            text: Some(s.into()),
            tool_calls: vec![],
            usage: Usage::default(),
        }
    }

    fn resp_call(id: &str, name: &str, args: &str) -> ChatResponse {
        ChatResponse {
            text: None,
            tool_calls: vec![ToolCallSpec {
                id: id.into(),
                name: name.into(),
                arguments: args.into(),
            }],
            usage: Usage::default(),
        }
    }

    async fn run_with(
        script: Vec<ChatResponse>,
        registry: ToolRegistry,
    ) -> (
        CsResult<RunOutcome>,
        Arc<Mutex<Vec<ChatRequest>>>,
        tempfile::TempDir,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let audit = Audit::create(dir.path(), "t").unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let provider = Arc::new(Scripted {
            script: Mutex::new(script.into_iter().collect()),
            seen: seen.clone(),
        });
        let harness = Harness::new(provider, registry, audit, "m".into(), 1_000_000, 60);
        let res = harness.run("任务").await;
        (res, seen, dir)
    }

    #[tokio::test]
    async fn dedup_rejects_same_call_and_executes_once() {
        let calls = Arc::new(Mutex::new(0u32));
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Echo {
            calls: calls.clone(),
            output: "out v1".into(),
        }));
        let script = vec![
            resp_call("1", "echo", "{\"q\":1}"),
            resp_call("2", "echo", "{\"q\":1}"), // 同 canonical → 拒绝
            resp_text("done"),
            resp_text("done"), // prose 降级
        ];
        let (res, _seen, _dir) = run_with(script, reg).await;
        let out = res.unwrap();
        assert!(out.answer.contains("done"));
        assert_eq!(*calls.lock().unwrap(), 1);
        assert_eq!(out.tool_calls, 1);
        assert!(out.report.degraded);
    }

    #[tokio::test]
    async fn zero_gain_streak_injects_steering() {
        let calls = Arc::new(Mutex::new(0u32));
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Echo {
            calls: calls.clone(),
            output: "same output".into(),
        }));
        let script = vec![
            resp_call("1", "echo", "{\"q\":1}"),
            resp_call("2", "echo", "{\"q\":2}"),
            resp_call("3", "echo", "{\"q\":3}"),
            resp_text("done"),
            resp_text("done"),
        ];
        let (res, seen, _dir) = run_with(script, reg).await;
        assert!(res.is_ok());
        assert_eq!(*calls.lock().unwrap(), 3);
        let requests = seen.lock().unwrap();
        let steering_in_4th = requests[3]
            .messages
            .iter()
            .any(|m| matches!(m, ChatMessage::User { content } if content.contains("无新信息")));
        assert!(steering_in_4th, "第 4 次请求应包含转向指令");
    }

    #[tokio::test]
    async fn fuse_after_five_no_progress_steps() {
        let script: Vec<_> = (0..5)
            .map(|i| resp_call(&i.to_string(), "nope", "{}"))
            .chain(std::iter::once(resp_text("never")))
            .collect();
        let (res, _seen, _dir) = run_with(script, ToolRegistry::new()).await;
        let err = res.unwrap_err();
        assert_eq!(err.code, LLM_FUSE);
        assert_eq!(err.exit_code(), 3);
    }

    #[tokio::test]
    async fn tool_errors_count_toward_no_progress_fuse() {
        // P004 T2.1 回归：千变参数的错误调用也必须计入熔断（旧实现完全不计数，可无限空转烧钱）
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(AlwaysErr));
        let script: Vec<_> = (0..5)
            .map(|i| resp_call(&format!("e{i}"), "always_err", &format!(r#"{{"n":{i}}}"#)))
            .chain(std::iter::once(resp_text("never")))
            .collect();
        let (res, _seen, _dir) = run_with(script, reg).await;
        let err = res.unwrap_err();
        assert_eq!(err.code, LLM_FUSE);
    }

    #[tokio::test]
    async fn prose_streak_resets_on_tool_turns() {
        // P004 T2.2 回归：工具轮之后 prose 计数归零——下一轮 prose 必须重新获得
        // submit_report 引导机会，而不是被旧计数直接降级。
        let calls = Arc::new(Mutex::new(0u32));
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Echo {
            calls: calls.clone(),
            output: "found in src/retry.rs".into(),
        }));
        let (res, seen, _dir) = run_with(
            vec![
                resp_text("先说一句"),         // prose 1 → 引导
                resp_call("t1", "echo", "{}"), // 工具轮 → 计数归零
                resp_text("又说一句"),         // prose → 必须再次引导（旧实现直接降级）
                resp_text("再摆烂"),           // prose 2 → 此时才降级
            ],
            reg,
        )
        .await;
        let out = res.unwrap();
        assert!(out.report.degraded); // 两次摆烂后最终降级 ✓
        let steers = seen
            .lock()
            .unwrap()
            .iter()
            .filter(|r| {
                r.messages.iter().any(|m| {
                    matches!(m, ChatMessage::User { content } if content.contains("submit_report"))
                })
            })
            .count();
        assert!(
            steers >= 2,
            "工具轮之后必须重新引导（实际引导 {steers} 次）"
        );
    }

    #[tokio::test]
    async fn prose_falls_back_to_degraded_report() {
        let (res, _seen, _dir) = run_with(
            vec![resp_text("直接回答"), resp_text("直接回答")],
            ToolRegistry::new(),
        )
        .await;
        let out = res.unwrap();
        assert!(out.report.degraded);
        assert_eq!(out.report.confidence, "low");
        assert!(out.answer.contains("直接回答"));
    }

    #[tokio::test]
    async fn submit_report_converges_with_validated_evidence() {
        let calls = Arc::new(Mutex::new(0u32));
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Echo {
            calls: calls.clone(),
            output: "found in src/retry.rs".into(),
        }));
        let submit_ok = resp_call(
            "s1",
            SUBMIT_TOOL,
            r#"{"answer":"重试在 retry.rs","findings":[{"statement":"核心函数","evidence":[{"file":"src/retry.rs","lines":"7-22"}]}],"dead_ends":[],"confidence":"high"}"#,
        );
        let script = vec![resp_call("1", "echo", "{\"q\":1}"), submit_ok];
        let (res, _seen, dir) = run_with(script, reg).await;
        let out = res.unwrap();
        assert!(!out.report.degraded);
        assert_eq!(out.report.confidence, "high");
        let ev = &out.report.findings[0].evidence[0];
        assert_eq!(ev.file, "src/retry.rs");
        // 互查：evidence.audit_seq 对应的审计行就是那次 tool_call（dir 保活审计文件）
        let seq = ev.audit_seq.expect("evidence 应带审计 seq");
        let content = std::fs::read_to_string(&out.audit_path).unwrap();
        let hits: Vec<Value> = content
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter(|v| v["seq"].as_u64() == Some(seq))
            .collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0]["kind"], "tool_call");
        assert_eq!(hits[0]["name"], "echo");
        drop(dir);
    }

    #[tokio::test]
    async fn submit_with_uncited_evidence_is_rejected_then_recovers() {
        let calls = Arc::new(Mutex::new(0u32));
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Echo {
            calls: calls.clone(),
            output: "found in src/retry.rs".into(),
        }));
        let bad = resp_call(
            "b1",
            SUBMIT_TOOL,
            r#"{"answer":"x","findings":[{"statement":"s","evidence":[{"file":"nope/none.rs","lines":"1"}]}],"confidence":"high"}"#,
        );
        let good = resp_call(
            "b2",
            SUBMIT_TOOL,
            r#"{"answer":"ok","findings":[{"statement":"s","evidence":[{"file":"src/retry.rs","lines":"7"}]}],"confidence":"high"}"#,
        );
        let script = vec![resp_call("1", "echo", "{\"q\":1}"), bad, good];
        let (res, _seen, _dir) = run_with(script, reg).await;
        let out = res.unwrap();
        assert!(!out.report.degraded);
        assert_eq!(out.report.findings[0].evidence[0].file, "src/retry.rs");
    }

    #[tokio::test]
    async fn recall_reads_audit_range_without_loss() {
        // 直接测 Audit::read_range 的不重不漏语义
        let dir = tempfile::tempdir().unwrap();
        let audit = Audit::create(dir.path(), "r").unwrap();
        for i in 1..=5 {
            audit
                .record("tool_call", &serde_json::json!({"i": i}))
                .unwrap();
        }
        let lines = audit.read_range(2, 4).unwrap();
        let seqs: Vec<u64> = lines.iter().map(|(s, _)| *s).collect();
        assert_eq!(seqs, vec![2, 3, 4]);
        let msg = context::recall_message(&lines);
        match &msg {
            ChatMessage::User { content } => {
                assert!(content.contains("#2"));
                assert!(content.contains("#4"));
                assert!(!content.contains("#1 "));
            }
            other => panic!("expect user, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn compaction_fires_when_window_exceeds_and_evicts() {
        // 小窗口（threshold=400 tokens≈1600 chars）+ 每回合 2k chars 唯一输出
        // → 第 5 回合起消息数破 8，必然发生真实驱逐
        let calls = Arc::new(Mutex::new(0u32));
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Echo {
            calls: calls.clone(),
            output: String::new(), // 每次由 args 决定，见下
        }));
        // Echo 输出固定，改用带状态的工具：每个不同 args 产生不同大输出
        struct BigEcho;
        #[async_trait::async_trait]
        impl Tool for BigEcho {
            fn name(&self) -> &'static str {
                "big_echo"
            }
            fn description(&self) -> String {
                "big echo".into()
            }
            fn parameters(&self) -> Value {
                serde_json::json!({"type": "object"})
            }
            async fn execute(&self, args: Value) -> CsResult<String> {
                let n = args.get("n").and_then(Value::as_u64).unwrap_or(1);
                Ok(format!("unique output #{n}: {}", "y".repeat(2000)))
            }
        }
        let mut reg2 = ToolRegistry::new();
        reg2.register(Box::new(BigEcho));

        let mut script = Vec::new();
        for i in 1..=7 {
            script.push(resp_call(
                &i.to_string(),
                "big_echo",
                &format!("{{\"n\":{i}}}"),
            ));
        }
        script.push(resp_text("done"));
        script.push(resp_text("done"));

        let dir = tempfile::tempdir().unwrap();
        let audit = Audit::create(dir.path(), "c").unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let provider = Arc::new(Scripted {
            script: Mutex::new(script.into_iter().collect()),
            seen: seen.clone(),
        });
        let harness = Harness::new(
            provider,
            reg2,
            audit,
            "m".into(),
            400, // threshold = 400 × 100% = 400 tokens ≈ 1600 chars
            100,
        );
        let outcome = harness.run("任务").await.unwrap();

        let compactions: Vec<u64> = {
            let content = std::fs::read_to_string(outcome.audit_path).unwrap();
            content
                .lines()
                .filter_map(|l| serde_json::from_str::<Value>(l).ok())
                .filter(|v| v["kind"] == "compaction")
                .filter_map(|v| v["evicted_count"].as_u64())
                .collect()
        };
        assert!(
            !compactions.is_empty(),
            "应发生真实压缩驱逐: {compactions:?}"
        );
        // 压缩后的请求应包含承上启下 handoff
        let requests = seen.lock().unwrap();
        let has_handoff = requests.iter().any(|r| {
            r.messages
                .iter()
                .any(|m| matches!(m, ChatMessage::User { content } if content.contains("承上启下")))
        });
        assert!(has_handoff, "压缩后应有 handoff 消息注入");
    }

    #[test]
    fn info_gain_keys_dedupe_identical_output() {
        let a = info_keys("src/retry.rs 的逻辑");
        let b = info_keys("src/retry.rs 的逻辑");
        assert_eq!(a.len(), b.len());
        assert!(a.iter().all(|k| b.contains(k)));
        assert!(a.iter().any(|k| k.contains("retry.rs")));
    }

    #[test]
    fn evidence_store_observes_citations() {
        let mut store = EvidenceStore::default();
        store.observe("输出提到 src/main.rs", 3);
        assert_eq!(store.cite_seq("src/main.rs"), Some(3));
    }

    #[test]
    fn evidence_exact_paths_with_spaces_and_cjk() {
        // FINDING-010 回归：含空格/中文的路径必须以精确形式存取
        let mut store = EvidenceStore::default();
        let spaced = "00 日记/2026 年 7 月 7 日.md";
        store.observe_exact(spaced, 7);
        assert_eq!(
            store.cite_seq(spaced),
            Some(7),
            "observe_exact 后应逐字命中"
        );

        // 切词器会把该路径劈成碎片，精确记录不受影响（这正是 read 精确存证存在的原因）
        store.observe("read 输出提到 00 日记/2026 年 7 月 7 日.md 的内容", 9);
        assert_eq!(
            store.cite_seq(spaced),
            Some(7),
            "碎片化观察不应破坏精确记录"
        );
    }
}
