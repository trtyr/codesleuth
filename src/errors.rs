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
pub const LLM_FUSE: CsCode = CsCode(2099);
// ---- CS3xxx 目标库 ----
pub const REPO_NOT_FOUND: CsCode = CsCode(3001);
pub const REPO_NOT_READABLE: CsCode = CsCode(3002);
pub const FENCE_DENIED: CsCode = CsCode(3003);
// ---- CS4xxx 索引 ----
pub const INDEX_NOT_AVAILABLE: CsCode = CsCode(4010);
pub const INDEX_BUILD_FAILED: CsCode = CsCode(4011);
pub const INDEX_STALE: CsCode = CsCode(4012);
pub const INDEX_CORRUPT: CsCode = CsCode(4013);
pub const INDEX_TIMEOUT: CsCode = CsCode(4014);
pub const INDEX_EMBED_FAILED: CsCode = CsCode(4015);
// ---- CS5xxx 内部 ----
pub const INTERNAL: CsCode = CsCode(5001);
pub const ENGINE_NOT_WIRED: CsCode = CsCode(5099);

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
}

impl CsError {
    pub fn new(code: CsCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
            retryable: false,
            source_text: None,
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

    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }
}

/// stderr 结构化报错（人读主行 + hint 行）。
pub fn report_error(e: &CsError) {
    eprintln!("{e}");
    if let Some(src) = &e.source_text {
        eprintln!("  ↳ 根因: {src}");
    }
    if let Some(hint) = &e.hint {
        eprintln!("  hint: {hint}");
    }
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
        let e = CsError::new(CONFIG_MISSING, "key 未找到")
            .with_hint("写入 ~/.codesleuth/config.toml 的 [llm].api_key")
            .with_source("read config.toml: no such file")
            .retryable();
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
}
