//! 嵌入输入组装（P003 精简版：描述层已移除）。
//!
//! A Raw       = chunk.text（面包屑+签名+docstring+函数体，裸代码）
//! B Composite = 白拿层（header）+ 标识符层（代码内关键标识符，去重封顶）——默认模式

use crate::vector::chunk::Chunk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedMode {
    Raw,
    Composite,
}

/// 抽取代码文本中的 ASCII 标识符（[A-Za-z_][A-Za-z0-9_]{3,}），去重保序，封顶 cap。
pub fn identifiers_of(text: &str, cap: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_alphabetic() || c == '_' {
            let mut j = i + 1;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            let token: String = chars[i..j].iter().collect();
            if token.len() >= 4 && !out.contains(&token) {
                out.push(token);
                if out.len() >= cap {
                    return out;
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// 组装嵌入输入。
pub fn compose_input(chunk: &Chunk, mode: EmbedMode) -> String {
    match mode {
        EmbedMode::Raw => chunk.text.clone(),
        EmbedMode::Composite => format!(
            "{}\n{}",
            chunk.header,
            identifiers_of(&chunk.text, 20).join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chunk() -> Chunk {
        Chunk {
            file: "src/fixture_py/retry.py".into(),
            line_start: 16,
            line_end: 30,
            symbol: "retry_with_backoff".into(),
            kind: "function".into(),
            language: "python".into(),
            header: "src/fixture_py/retry.py › retry_with_backoff（function）\n装饰器：按指数退避重试。".into(),
            text: "src/fixture_py/retry.py › retry_with_backoff（function）\n装饰器：按指数退避重试。\ndef retry_with_backoff(func):\n    raise RetryError()".into(),
            text_hash: "hash".into(),
        }
    }

    #[test]
    fn raw_mode_returns_full_text() {
        let c = sample_chunk();
        assert_eq!(compose_input(&c, EmbedMode::Raw), c.text);
    }

    #[test]
    fn composite_excludes_body_but_keeps_identifiers() {
        let out = compose_input(&sample_chunk(), EmbedMode::Composite);
        assert!(out.contains("retry_with_backoff（function）"));
        assert!(out.contains("RetryError"));
        assert!(
            !out.contains("def retry_with_backoff(func):"),
            "函数体不进复合体"
        );
    }

    #[test]
    fn identifiers_dedupe_and_cap() {
        let ids = identifiers_of("alpha beta alpha gamma_1 x zz ToolCallSpec long_enough", 3);
        assert_eq!(ids, vec!["alpha", "beta", "gamma_1"]);
    }
}
