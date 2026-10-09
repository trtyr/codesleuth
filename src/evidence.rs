//! 证据库 + 信息增量键（P004 T6.2 从 harness.rs 角色分离迁出）。
//!
//! harness 主循环只管编排；「什么算证据、怎么记账」的语义集中在本模块。
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

/// 信息增量键：路径形 token + 内容指纹（solution-map §3 增量记账）。
pub fn info_keys(output: &str) -> Vec<String> {
    let mut keys: HashSet<String> = HashSet::new();
    for tok in output.split(|c: char| c.is_whitespace() || "[]()<>\"'`,;".contains(c)) {
        let t = tok.trim_matches(|c| c == '.' || c == ':');
        if path_like(t) {
            keys.insert(t.to_string());
        }
    }
    let digest = Sha256::digest(output.as_bytes());
    let hex: String = digest[..8].iter().map(|b| format!("{b:02x}")).collect();
    keys.insert(hex);
    keys.into_iter().collect()
}

fn path_like(t: &str) -> bool {
    t.len() >= 3 && (t.contains('/') || t.contains('.')) && !t.contains("://")
}

/// 证据库：会话内真实观察过的路径 → 首次观察的审计 seq（submit_report 校验 + 互查锚点）。
#[derive(Default)]
pub struct EvidenceStore {
    paths: HashMap<String, u64>,
}

impl EvidenceStore {
    /// 精确路径入库（read 工具专用，FINDING-010）：模型传入什么存什么，引用时逐字匹配。
    /// 不走空白切词——含空格/中文的路径会被劈成碎片。
    pub fn observe_exact(&mut self, path: &str, seq: u64) {
        self.paths.entry(path.to_string()).or_insert(seq);
    }

    /// 已观察路径清单（FINDING-012 零 findings 打回用）。只列真实读过内容的路径。
    pub fn observed_paths(&self) -> Vec<String> {
        self.paths.keys().cloned().collect()
    }

    // P007 R1.1：删除 observe()——通用切词入库让 grep/find_files 输出中被「提及」的路径
    // 充当 evidence，击穿「必须真实读过」门禁。paths 的唯一喂入口是 observe_exact（真实读到）。

    pub fn cite_seq(&self, path: &str) -> Option<u64> {
        self.paths.get(path).copied()
    }
}
