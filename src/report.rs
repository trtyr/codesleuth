//! 侦察报告（solution-map §7）：机器可读 schema（版本化）+ 人类渲染。
//! 降级路径：模型未走 submit_report 时由 prose 构造（degraded=true，诚实标注）。

use crate::errors::{CsError, CsResult};
use serde::{Deserialize, Serialize};

pub const REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    pub file: String,
    pub lines: String,
    /// 证据首次观察到的审计行 seq（报告 ↔ 审计互查锚点；degraded 时可能为 None）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_seq: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    pub statement: String,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ReportStats {
    pub turns: u32,
    pub tool_calls: u32,
    pub duration_ms: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub report_schema_version: u32,
    pub task: String,
    pub answer: String,
    pub findings: Vec<Finding>,
    pub dead_ends: Vec<String>,
    pub confidence: String,
    /// true = 模型未走 submit_report，由 prose 降级构造（诚实标注）。
    pub degraded: bool,
    pub stats: ReportStats,
}

impl Report {
    pub fn degraded_prose(task: &str, answer: &str, stats: ReportStats) -> Self {
        Self {
            report_schema_version: REPORT_SCHEMA_VERSION,
            task: task.to_string(),
            answer: answer.to_string(),
            findings: vec![],
            dead_ends: vec![],
            confidence: "low".into(),
            degraded: true,
            stats,
        }
    }

    pub fn validate(&self) -> CsResult<()> {
        // P007 R3.3：报告内容校验不再借用 INDEX_BUILD_FAILED（误导为索引故障、exit 5）——
        // 语义即输出契约（CS2005，与 harness 契约校验同码）
        if !matches!(self.confidence.as_str(), "high" | "medium" | "low") {
            return Err(CsError::new(
                crate::errors::OUTPUT_CONTRACT,
                format!("非法置信度: {}", self.confidence),
            )
            .with_hint("只允许 high | medium | low"));
        }
        if self.answer.trim().is_empty() {
            return Err(CsError::new(crate::errors::OUTPUT_CONTRACT, "answer 为空"));
        }
        Ok(())
    }

    /// 人类渲染（确定性模板，可快照测试）。
    pub fn render_human(&self) -> String {
        let mut out = String::new();
        out.push_str("# 侦察报告\n\n");
        out.push_str(&format!("任务：{}\n\n", self.task));
        out.push_str("## 结论\n");
        out.push_str(&format!("{}\n\n", self.answer));
        if self.findings.is_empty() {
            // P007 R3.4：零 findings + dead_ends 的合法非降级报告不再被误标「降级」
            let note = if self.degraded {
                "（无结构化发现——降级报告）"
            } else {
                "（无结构化发现；参见死胡同）"
            };
            out.push_str(&format!("## 证据列表\n{note}\n\n"));
        } else {
            out.push_str("## 证据列表\n");
            for (i, f) in self.findings.iter().enumerate() {
                out.push_str(&format!("{}. {}\n", i + 1, f.statement));
                for e in &f.evidence {
                    let seq = e
                        .audit_seq
                        .map(|s| format!("审计 #{s}"))
                        .unwrap_or_default();
                    out.push_str(&format!("   - {}:{}（{seq}）\n", e.file, e.lines));
                }
            }
            out.push('\n');
        }
        if self.dead_ends.is_empty() {
            out.push_str("## 死胡同\n无\n\n");
        } else {
            out.push_str("## 死胡同\n");
            for d in &self.dead_ends {
                out.push_str(&format!("- {d}\n"));
            }
            out.push('\n');
        }
        let degraded_note = if self.degraded {
            "（降级：模型未走结构化提交）"
        } else {
            ""
        };
        out.push_str(&format!(
            "## 置信度\n{}{degraded_note}\n\n",
            self.confidence
        ));
        out.push_str(&format!(
            "## 统计\nturns={} · tool_calls={} · duration={}ms · tokens={}\n",
            self.stats.turns,
            self.stats.tool_calls,
            self.stats.duration_ms,
            self.stats.total_tokens
        ));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn sample() -> Report {
        Report {
            report_schema_version: REPORT_SCHEMA_VERSION,
            task: "重试逻辑在哪".into(),
            answer: "在 src/retry.rs".into(),
            findings: vec![Finding {
                statement: "核心重试函数".into(),
                evidence: vec![Evidence {
                    file: "src/retry.rs".into(),
                    lines: "7-22".into(),
                    audit_seq: Some(5),
                }],
            }],
            dead_ends: vec!["config.rs 无重试".into()],
            confidence: "high".into(),
            degraded: false,
            stats: ReportStats {
                turns: 3,
                tool_calls: 4,
                duration_ms: 1234,
                total_tokens: 5678,
            },
        }
    }

    #[test]
    fn schema_version_roundtrip() {
        let json = serde_json::to_string(&sample()).unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["report_schema_version"], 1);
        let back: Report = serde_json::from_str(&json).unwrap();
        assert_eq!(back, sample());
    }

    #[test]
    fn human_render_snapshot() {
        let expected = "# 侦察报告\n\n任务：重试逻辑在哪\n\n## 结论\n在 src/retry.rs\n\n## 证据列表\n1. 核心重试函数\n   - src/retry.rs:7-22（审计 #5）\n\n## 死胡同\n- config.rs 无重试\n\n## 置信度\nhigh\n\n## 统计\nturns=3 · tool_calls=4 · duration=1234ms · tokens=5678\n";
        assert_eq!(sample().render_human(), expected);
    }

    #[test]
    fn degraded_report_marks_itself() {
        let r = Report::degraded_prose("t", "纯文本回答", ReportStats::default());
        assert!(r.degraded);
        assert_eq!(r.confidence, "low");
        assert!(r.render_human().contains("降级"));
    }

    #[test]
    fn validate_rejects_bad_confidence() {
        let mut r = sample();
        r.confidence = "超高".into();
        let err = r.validate().unwrap_err();
        // P007 R3.3：报告校验错误码 = OUTPUT_CONTRACT，不再误用 INDEX_BUILD_FAILED
        assert_eq!(err.code, crate::errors::OUTPUT_CONTRACT);
        r.confidence = "high".into();
        assert!(r.validate().is_ok());
    }

    #[test]
    fn zero_findings_with_dead_ends_is_not_rendered_as_degraded() {
        // P007 R3.4 回归：合法零 findings + dead_ends 报告（degraded=false）
        // 不再被渲染成「降级报告」
        let mut r = sample();
        r.findings.clear();
        assert!(!r.degraded);
        let out = r.render_human();
        assert!(out.contains("无结构化发现；参见死胡同"));
        assert!(!out.contains("降级报告"));
    }
}
