//! 向量混合检索（P003）：嵌入管线 + codegraph 切块适配器 + SQLite 索引存储 + 描述生成 + 复合体组装 + 召回引擎。
//!
//! 分层（检索设计稿）：embed = Qwen3 嵌入客户端；chunk = codegraph 符号表边界真源 + 三层规则；
//! store = SQLite 单文件真相（text_hash 增量）；describe = MiniMax-M3 关系感知描述（军规：thinking off/并发3）；
//! compose = 三层复合体组装（A 裸/B 复合/C 含描述）；recall = HNSW 影子图召回引擎；build = 索引构建编排。

pub mod build;
pub mod chunk;
pub mod compose;
pub mod embed;
pub mod recall;
pub mod repomap;
pub mod store;

pub use build::{BuildReport, build_vector_index};
pub use chunk::{Chunk, SymbolRow, plan_chunks};
pub use compose::{EmbedMode, compose_input};
pub use embed::{EmbedClient, instruct_query};
pub use recall::RecallEngine;
pub use store::{VectorStore, fingerprint, index_path};
