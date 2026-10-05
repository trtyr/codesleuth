//! 上下文管理（D008 立骨架，D013 升 v2）：
//! 阈值 = 模型窗口 × 百分比（物理口径，默认 1M × 60%）；
//! 触发后三段式：①持久化（账本落盘）→ ②承上启下 handoff（先于压缩生成，确定性零 LLM）→ ③压缩。
//! recall 从审计原文续读（不重不漏）。

use crate::llm::ChatMessage;

/// 压缩时保留的最近消息条数。
pub const KEEP_RECENT: usize = 6;

fn msg_chars(m: &ChatMessage) -> usize {
    match m {
        ChatMessage::System { content } | ChatMessage::User { content } => content.len(),
        ChatMessage::Assistant {
            content,
            tool_calls,
        } => {
            content.as_deref().map(str::len).unwrap_or(0)
                + tool_calls.iter().map(|t| t.arguments.len()).sum::<usize>()
        }
        ChatMessage::Tool { content, .. } => content.len(),
    }
}

/// 估算当前窗口 token（chars/4 启发式）。
pub fn estimate_tokens(messages: &[ChatMessage]) -> usize {
    messages.iter().map(msg_chars).sum::<usize>() / 4
}

/// 压缩阈值 = 模型上下文窗口 × 百分比（D013；percent > 100 按 100 封顶）。
pub fn compact_threshold_tokens(model_context_tokens: u64, percent: u64) -> usize {
    (model_context_tokens.saturating_mul(percent.min(100)) / 100) as usize
}

pub fn should_compact(messages: &[ChatMessage], max_tokens: usize) -> bool {
    estimate_tokens(messages) >= max_tokens
}

#[derive(Debug, Clone, PartialEq)]
pub struct Compaction {
    pub evicted_count: usize,
}

/// 承上启下 handoff（D013）：先于压缩生成的衔接物——
/// 任务重述 + 驱逐范围与 recall 指引 + 下一步指引。
/// 确定性拼装，零 LLM。
pub fn build_handoff(task: &str, audit_from: u64, audit_to: u64) -> String {
    format!(
        "[承上启下 · 压缩交接]\n\
         任务：{task}\n\n\
         已驱逐早期原始消息（审计 seq {audit_from}..={audit_to}；全量原文用 recall {{\"from\": .., \"to\": ..}} 续读，不重不漏）。\
         下一步：基于已读内容继续未完成线索；不要重做已完成的检索。"
    )
}

/// 确定性压缩：保留头部（system+task）与最近 K 条，中间替换为 handoff 通知。零 LLM 调用。
pub fn compact(
    messages: Vec<ChatMessage>,
    handoff_body: String,
    keep_recent: usize,
) -> (Vec<ChatMessage>, Compaction) {
    let total = messages.len();
    let head = 2.min(total); // system + task
    let tail = keep_recent.min(total.saturating_sub(head));
    let evict_end = total.saturating_sub(tail);
    // 消息对感知（P004 T1.2）：裁剪边界若落在 tool 消息上，其 assistant(tool_calls) 搭档
    // 即将被驱逐——边界回退把整对留在保留侧，否则下游 API 以 400 拒收孤儿 tool 消息。
    let mut evict_end = evict_end;
    while evict_end > head && matches!(messages[evict_end], ChatMessage::Tool { .. }) {
        evict_end -= 1;
    }
    let evicted_count = evict_end.saturating_sub(head);
    if evicted_count == 0 {
        return (messages, Compaction { evicted_count: 0 });
    }
    let mut out: Vec<ChatMessage> = messages[..head].to_vec();
    out.push(ChatMessage::User {
        content: handoff_body,
    });
    out.extend_from_slice(&messages[evict_end..]);
    (out, Compaction { evicted_count })
}

/// recall 结果的消息化包装（D008 钻取：原文逐行 seq 标注，不重不漏）。
pub fn recall_message(lines: &[(u64, serde_json::Value)]) -> ChatMessage {
    let mut body = String::from("[recall] 审计原文（按 seq 升序，不重不漏）：\n");
    for (seq, v) in lines {
        body.push_str(&format!("#{seq} {}\n", v));
    }
    if lines.is_empty() {
        body.push_str("（该范围无审计行）\n");
    }
    ChatMessage::User { content: body }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::ToolCallSpec;

    fn user(s: &str) -> ChatMessage {
        ChatMessage::User { content: s.into() }
    }

    fn big_user(tag: &str) -> ChatMessage {
        user(&format!("{tag}{}", "x".repeat(2000)))
    }

    #[test]
    fn threshold_is_percent_of_context() {
        assert_eq!(compact_threshold_tokens(1_000_000, 60), 600_000);
        assert_eq!(compact_threshold_tokens(100_000, 50), 50_000);
        assert_eq!(compact_threshold_tokens(100_000, 150), 100_000); // 封顶 100%
    }

    #[test]
    fn threshold_triggers_by_size() {
        let small = vec![user("hi")];
        assert!(!should_compact(&small, 24_000));
        let big: Vec<ChatMessage> = (0..50).map(|_| big_user("t")).collect();
        assert!(should_compact(&big, 24_000));
    }

    #[test]
    fn handoff_recall_starts_at_first_audit_seq() {
        // P005 R4.1：审计 seq 从 1 起号，handoff 指引的 recall 范围必须从 1 开始
        let h = build_handoff("找重试", 1, 7);
        assert!(h.contains("seq 1..=7"), "handoff 应包含 seq 1 起点: {h}");
    }

    #[test]
    fn compact_keeps_head_and_recent_keeps_handoff() {
        let mut msgs = vec![user("SYSTEM"), user("TASK")];
        for i in 0..8 {
            msgs.push(big_user(&format!("m{i} ")));
        }
        let handoff = build_handoff("找重试", 2, 42);
        let (out, info) = compact(msgs, handoff, 6);
        assert_eq!(info.evicted_count, 2);
        // head 2 + handoff 1 + 最近 6 = 9
        assert_eq!(out.len(), 9);
        assert_eq!(out[0], user("SYSTEM"));
        assert_eq!(out[1], user("TASK"));
        let notice = match &out[2] {
            ChatMessage::User { content } => content.clone(),
            other => panic!("expect user notice, got {other:?}"),
        };
        assert!(notice.contains("承上启下"));
        assert!(notice.contains("不要重做"));
        assert!(notice.contains("recall"));
        assert!(notice.contains("任务：找重试"));
    }

    #[test]
    fn nothing_to_evict_is_noop() {
        let msgs = vec![user("a"), user("b"), user("c")];
        let (out, info) = compact(msgs.clone(), "handoff".into(), 6);
        assert_eq!(info.evicted_count, 0);
        assert_eq!(out, msgs);
    }

    #[test]
    fn compact_never_splits_tool_call_pair() {
        // P004 T1.2 回归：裁剪边界恰好落在 tool 消息上时，其 assistant(tool_calls) 搭档
        // 不能被驱逐——旧实现会把 pair 拆散，下游 API 以 400 拒收孤儿 tool 消息。
        let mut msgs = vec![user("SYSTEM"), user("TASK")];
        for i in 0..8 {
            msgs.push(big_user(&format!("m{i} ")));
        }
        // 让边界正好切在 assistant 与它的 tool 结果之间：尾部倒数第 2 条是 assistant(tool_calls)，
        // 最后一条是它的 tool 结果，keep_recent=1 时旧实现会把 tool 单独留在保留侧。
        msgs.push(ChatMessage::Assistant {
            content: None,
            tool_calls: vec![ToolCallSpec {
                id: "call-1".into(),
                name: "read".into(),
                arguments: "{}".into(),
            }],
        });
        msgs.push(ChatMessage::Tool {
            call_id: "call-1".into(),
            content: "file content".into(),
        });
        let (out, info) = compact(msgs, "handoff".into(), 1);
        // 保留侧尾部：assistant 与 tool 必须同在，且 tool 不能是保留区第一条（孤儿）
        let tail_has_pair = out.windows(2).any(|w| {
            matches!(&w[0], ChatMessage::Assistant { tool_calls, .. } if !tool_calls.is_empty())
                && matches!(&w[1], ChatMessage::Tool { .. })
        });
        assert!(tail_has_pair, "tool 调用对必须整对存活");
        if let Some(ChatMessage::Tool { .. }) = out.first() {
            panic!("保留区不得以孤儿 tool 消息开头");
        }
        assert!(info.evicted_count > 0);
    }

    #[test]
    fn estimate_counts_tool_args() {
        let m = ChatMessage::Assistant {
            content: None,
            tool_calls: vec![ToolCallSpec {
                id: "1".into(),
                name: "read".into(),
                arguments: "x".repeat(400),
            }],
        };
        assert!(estimate_tokens(&[m]) >= 100);
    }
}
