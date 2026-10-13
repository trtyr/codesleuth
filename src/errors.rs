//! 错误码体系（D003.1）。
//!
//! 段位：CS1xxx 用户/配置 · CS2xxx 上游 LLM · CS3xxx 目标库 IO/围栏 · CS4xxx 索引 · CS5xxx 内部。
//! 每个错误 = 码 + 人话 + 修复建议 + 是否可重试；exit code 由码段决定（见 cli-and-output.md）。

use std::fmt;

/// 结构化错误码。段位决定 exit code，细码定位问题。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsCode(pub u16);

impl CsCode {
    pub const fn exit_code(self) -> i32 {
        match self.0 {
            1000..=1009 => 1, // 用法错误
            1010..=1099 => 2, // 配置/凭据
            2000..=2999 => 3, // 上游 LLM（含熔断）
            3000..=3999 => 4, // 目标库
            4000..=4999 => 5, // 索引
            _ => 6,           // 内部
        }
    }
}

impl fmt::Display for CsCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CS{:04}", self.0)
    }
}

// ---- CS1xxx 用户 / 配置 ----
pub const USER_INPUT: CsCode = CsCode(1001);
pub const CONFIG_MISSING: CsCode = CsCode(1011);
pub const CONFIG_INVALID: CsCode = CsCode(1012);
// ---- CS2xxx 上游 LLM ----
pub const LLM_UNREACHABLE: CsCode = CsCode(2001);
pub const LLM_RATE_LIMITED: CsCode = CsCode(2002);
pub const LLM_SERVER: CsCode = CsCode(2003);
pub const LLM_BAD_RESPONSE: CsCode = CsCode(2004);
/// D021（C4）：输出契约违约——`--require` 声明的必需标记经一轮修复仍缺失（重试/换模型可能好）。
pub const OUTPUT_CONTRACT: CsCode = CsCode(2005);
pub const LLM_FUSE: CsCode = CsCode(2099);
// ---- CS3xxx 目标库 ----
pub const REPO_NOT_FOUND: CsCode = CsCode(3001);
pub const REPO_NOT_READABLE: CsCode = CsCode(3002);
pub const FENCE_DENIED: CsCode = CsCode(3003);
// ---- CS4xxx 索引 ----
pub const INDEX_NOT_AVAILABLE: CsCode = CsCode(4010);
pub const INDEX_BUILD_FAILED: CsCode = CsCode(4011);
// 4012 INDEX_STALE / 4013 INDEX_CORRUPT 预留段位（P005 R6.2 清理零引用常量，需要时再启用）
pub const INDEX_TIMEOUT: CsCode = CsCode(4014);
pub const INDEX_EMBED_FAILED: CsCode = CsCode(4015);
pub const INDEX_LOCKED: CsCode = CsCode(4016);
// ---- CS5xxx 内部 ----
pub const INTERNAL: CsCode = CsCode(5001);
// 5099 ENGINE_NOT_WIRED 预留段位（P005 R6.2 清理零引用常量）

/// EN-17/D032：稳定失败原因码——机器可读失败分类，编排层按码分流
/// （parse_fail 原地重试 / truncated 提示输出约束 / fuse 换切分策略）。
/// `as_str()` 输出即对外稳定字符串契约（serde 序列化同形）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureReason {
    /// LLM 响应解析失败（含自修轮后仍失败）
    ParseFail,
    /// 连续无进展熔断（CS2099）
    Fuse,
    /// 宣称完成但零产出/未按通道交付（落盘补轮烧尽，EN-16）
    ZeroWrite,
    /// 输出契约违约（有产出但不合格：缺标记/证据不足）
    EvidenceShort,
    /// LLM 输出被截断（finish_reason=length）
    Truncated,
    /// 等待超时（锁等）
    Timeout,
}

impl FailureReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            FailureReason::ParseFail => "parse_fail",
            FailureReason::Fuse => "fuse",
            FailureReason::ZeroWrite => "zero_write",
            FailureReason::EvidenceShort => "evidence_short",
            FailureReason::Truncated => "truncated",
            FailureReason::Timeout => "timeout",
        }
    }
}

/// 一等错误：码 + 消息 + 修复建议 + 可重试标记。
#[derive(Debug, thiserror::Error)]
#[error("{code}: {message}")]
pub struct CsError {
    pub code: CsCode,
    pub message: String,
    pub hint: Option<String>,
    pub retryable: bool,
    /// 根因链（P004 T3）：内层错误原文，供程序化溯源；人话仍在 message。
    /// 字段名避开 `source`：thiserror 会把名为 source 的字段自动当 Error::source()，String 不满足。
    pub source_text: Option<String>,
    /// EN-17/D032：稳定失败原因码——机器可读分类（None = 无细分必要，不伪造）。
    pub reason: Option<FailureReason>,
}

impl CsError {
    pub fn new(code: CsCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
            retryable: false,
            source_text: None,
            reason: None,
        }
    }

    /// 挂根因链：map_err 时把内层错误原文存 source_text，不只在 message 里人话拼接。
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source_text = Some(source.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    // P007 R4.5：retryable() 单参构建器已删（生产零调用，全仓统一 with_retryable(bool)）

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    /// EN-17/D032：挂稳定失败原因码（机器可读分类）。
    pub fn with_reason(mut self, reason: FailureReason) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }
}

/// stderr 结构化报错（人读主行 + hint 行 + 原因码尾行）。
pub fn report_error(e: &CsError) {
    eprintln!("{e}");
    if let Some(src) = &e.source_text {
        eprintln!("  ↳ 根因: {src}");
    }
    if let Some(hint) = &e.hint {
        eprintln!("  hint: {hint}");
    }
    // EN-17/D032 人读轨：稳定原因码尾行（宽松编排层按行首 CS_REASON= 提取）
    if let Some(r) = &e.reason {
        eprintln!("CS_REASON={}", r.as_str());
    }
}

/// 编排层结构化错误 JSON（D026，EN-10）：`--output-format json` 时错误体走单行 JSON
/// 独立通道——外层脚本按行首 `{"code":` 稳定提取 CS 码，不再被 tracing
/// ANSI/时间戳/统计行污染（纯函数便于测试；打印见 report_error_structured）。
pub fn structured_error_json(e: &CsError) -> serde_json::Value {
    let mut row = serde_json::json!({
        "code": e.code.to_string(),
        "message": e.message,
        "exit_code": e.exit_code(),
        "retryable": e.retryable,
    });
    if let Some(h) = &e.hint {
        row["hint"] = serde_json::Value::String(h.clone());
    }
    if let Some(s) = &e.source_text {
        row["source"] = serde_json::Value::String(s.clone());
    }
    // EN-17/D032 机器轨：稳定原因码（Some 时出现，缺席不占位——同 hint/source 风格）
    if let Some(r) = &e.reason {
        row["reason"] = serde_json::Value::String(r.as_str().to_string());
    }
    row
}

/// 结构化错误通道打印：单行 JSON 到 stderr，独立于 tracing 输出。
pub fn report_error_structured(e: &CsError) {
    eprintln!("{}", structured_error_json(e));
}

pub type CsResult<T> = Result<T, CsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_follow_segment() {
        assert_eq!(USER_INPUT.exit_code(), 1);
        assert_eq!(CONFIG_MISSING.exit_code(), 2);
        assert_eq!(LLM_RATE_LIMITED.exit_code(), 3);
        assert_eq!(FENCE_DENIED.exit_code(), 4);
        assert_eq!(INDEX_BUILD_FAILED.exit_code(), 5);
        assert_eq!(INTERNAL.exit_code(), 6);
    }

    #[test]
    fn display_includes_code_and_message() {
        let e = CsError::new(LLM_UNREACHABLE, "网关连不上");
        assert_eq!(e.to_string(), "CS2001: 网关连不上");
    }

    #[test]
    fn builder_fluency() {
        // P007 R4.5：retryable() 已删，统一 with_retryable(bool)
        let e = CsError::new(CONFIG_MISSING, "key 未找到")
            .with_hint("写入 ~/.codesleuth/config.toml 的 [llm].api_key")
            .with_source("read config.toml: no such file")
            .with_retryable(true);
        assert!(e.retryable);
        assert_eq!(
            e.hint.as_deref(),
            Some("写入 ~/.codesleuth/config.toml 的 [llm].api_key")
        );
        assert_eq!(
            e.source_text.as_deref(),
            Some("read config.toml: no such file")
        );
    }

    /// D026（EN-10）：结构化错误通道——code/exit_code 必带，hint/source 缺席不占位
    #[test]
    fn structured_error_json_carries_code_and_optional_fields() {
        let e = CsError::new(INDEX_LOCKED, "等待超时")
            .with_hint("可用 --lock-timeout 调高")
            .with_source("flock: timeout");
        let v = structured_error_json(&e);
        assert_eq!(v["code"], "CS4016");
        assert_eq!(v["exit_code"], 5);
        assert_eq!(v["hint"], "可用 --lock-timeout 调高");
        assert_eq!(v["source"], "flock: timeout");
        assert_eq!(v["retryable"], false);
        let plain = structured_error_json(&CsError::new(USER_INPUT, "缺任务"));
        assert_eq!(plain["code"], "CS1001");
        assert!(plain.get("hint").is_none(), "缺席字段不占位");
        assert!(plain.get("source").is_none(), "缺席字段不占位");
        assert!(plain.get("reason").is_none(), "缺席字段不占位");
    }

    /// EN-17/D032：稳定原因码——as_str 映射表全值 + json 通道带 reason（Some 时出现）
    #[test]
    fn failure_reason_stable_codes_and_json_channel() {
        assert_eq!(FailureReason::ParseFail.as_str(), "parse_fail");
        assert_eq!(FailureReason::Fuse.as_str(), "fuse");
        assert_eq!(FailureReason::ZeroWrite.as_str(), "zero_write");
        assert_eq!(FailureReason::EvidenceShort.as_str(), "evidence_short");
        assert_eq!(FailureReason::Truncated.as_str(), "truncated");
        assert_eq!(FailureReason::Timeout.as_str(), "timeout");

        let e = CsError::new(LLM_RATE_LIMITED, "429").with_reason(FailureReason::Timeout);
        assert_eq!(structured_error_json(&e)["reason"], "timeout");
        // serde 序列化同形（对外稳定字符串契约）
        assert_eq!(
            serde_json::to_value(FailureReason::EvidenceShort).unwrap(),
            "evidence_short"
        );
    }
}
