//! 配置加载链（topics/cli-and-output.md）：
//! CLI 参数 > 项目配置（.codesleuth/config.toml，兼容 ./codesleuth.toml）> 全局配置（~/.codesleuth/config.toml）> 默认值。
//! 环境变量层已整体移除（2026-10-05 用户裁决）：一个配置文件管一切，不拉屎。

use crate::errors::{CONFIG_INVALID, CONFIG_MISSING, CsError, CsResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};


/// 配置文件形态（全部 Option，缺席 = 不覆盖）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileConfig {
    #[serde(default)]
    pub llm: FileLlm,
    #[serde(default)]
    pub context: FileContext,
    #[serde(default)]
    pub vector: FileVector,
    #[serde(default)]
    pub behavior: FileBehavior,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileVector {
    pub embed_model: Option<String>,
    pub embed_dims: Option<u32>,
    pub embed_mode: Option<String>,
    pub repomap_budget: Option<usize>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileBehavior {
    /// 检索型任务默认关思考；true = 开。
    pub thinking_on: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileContext {
    pub model_context_tokens: Option<u64>,
    pub compact_at_percent: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileLlm {
    pub base_url: Option<String>,
    /// 明文直配（2026-10-05 归位：API Key 就住在配置文件里）。
    pub api_key: Option<String>,
    pub model: Option<String>,
}

/// 运行时生效配置。
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub llm: LlmConfig,
    pub context: ContextConfig,
    pub vector: VectorConfig,
    pub thinking_disabled: bool,
}

/// 向量层参数（原散落 env，2026-10-05 收编进配置链）。
#[derive(Debug, Clone, PartialEq)]
pub struct VectorConfig {
    pub embed_model: String,
    pub embed_dims: u32,
    /// "raw" | "composite"
    pub embed_mode: String,
    pub repomap_budget: usize,
    /// 嵌入专用端点（None = 跟随 [llm].base_url）：chat 与 embedding 常是两家供应商。
    pub base_url: Option<String>,
    /// 嵌入专用密钥（None = 跟随主 api_key）。
    pub api_key: Option<String>,
}

/// 上下文策略（D013）：压缩阈值 = 模型窗口 × 百分比。
#[derive(Debug, Clone, PartialEq)]
pub struct ContextConfig {
    pub model_context_tokens: u64,
    pub compact_at_percent: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
    /// 配置文件直配密钥（存在且非空则优先于 env 间接）。
    pub api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                base_url: "https://api.openai.com/v1".into(),
                model: "gpt-4o-mini".into(),
                api_key: None,
            },
            context: ContextConfig {
                model_context_tokens: 1_000_000,
                compact_at_percent: 60,
            },
            vector: VectorConfig {
                embed_model: "Qwen/Qwen3-Embedding-8B".into(),
                embed_dims: 1024,
                embed_mode: "composite".into(),
                repomap_budget: 24_000,
                base_url: None,
                api_key: None,
            },
            thinking_disabled: true,
        }
    }
}

/// CLI 覆盖（与 clap 解耦）。
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub base_url: Option<String>,
    pub model: Option<String>,
}

/// 全局配置（用户拍板 2026-10-05）：~/.codesleuth/config.toml——一切全局东西的家。
pub fn global_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".codesleuth").join("config.toml"))
}

/// 全局产物根（reports/audit/ledger/eval）——同一个家。
pub fn global_state_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".codesleuth"))
}

/// 项目配置：优先 <project>/.codesleuth/config.toml，兼容旧 ./codesleuth.toml。
pub fn project_config_path() -> PathBuf {
    PathBuf::from(".codesleuth/config.toml")
}

pub fn project_config_path_legacy() -> PathBuf {
    PathBuf::from("codesleuth.toml")
}

/// 解析单个配置文件。
pub fn parse_file(path: &Path) -> CsResult<FileConfig> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        CsError::new(CONFIG_INVALID, format!("读取失败: {e}"))
            .with_hint(format!("文件: {}", path.display()))
    })?;
    toml::from_str(&text).map_err(|e| {
        CsError::new(CONFIG_INVALID, format!("解析失败: {e}"))
            .with_hint(format!("文件: {}", path.display()))
    })
}

/// 读取真实配置（全局文件 + 项目文件 + CLI 覆盖；环境变量层已移除）。
pub fn load(overrides: CliOverrides) -> CsResult<Config> {
    let global = global_config_path().filter(|p| p.exists());
    let project = {
        let p = project_config_path();
        p.exists().then_some(p)
    };
    let project = if project.is_some() {
        project
    } else {
        let legacy = project_config_path_legacy();
        legacy.exists().then_some(legacy)
    };
    load_layered(global.as_deref(), project.as_deref(), overrides)
}

/// 纯函数加载链（可测）：默认值 ← 全局 ← 项目 ← CLI。
pub fn load_layered(
    global: Option<&Path>,
    project: Option<&Path>,
    cli: CliOverrides,
) -> CsResult<Config> {
    let mut cfg = Config::default();
    for path in [global, project].into_iter().flatten() {
        merge_file(&mut cfg, parse_file(path)?);
    }
    merge_cli(&mut cfg, cli);
    Ok(cfg)
}

fn merge_file(cfg: &mut Config, fc: FileConfig) {
    if let Some(v) = fc.llm.api_key {
        cfg.llm.api_key = Some(v);
    }
    if let Some(v) = fc.vector.embed_model {
        cfg.vector.embed_model = v;
    }
    if let Some(v) = fc.vector.embed_dims {
        cfg.vector.embed_dims = v;
    }
    if let Some(v) = fc.vector.embed_mode {
        cfg.vector.embed_mode = v;
    }
    if let Some(v) = fc.vector.repomap_budget {
        cfg.vector.repomap_budget = v;
    }
    if let Some(v) = fc.vector.base_url {
        cfg.vector.base_url = Some(v);
    }
    if let Some(v) = fc.vector.api_key {
        cfg.vector.api_key = Some(v);
    }
    if let Some(v) = fc.behavior.thinking_on {
        cfg.thinking_disabled = !v;
    }
    if let Some(v) = fc.llm.base_url {
        cfg.llm.base_url = v;
    }
    if let Some(v) = fc.llm.model {
        cfg.llm.model = v;
    }
    if let Some(v) = fc.context.model_context_tokens {
        cfg.context.model_context_tokens = v;
    }
    if let Some(v) = fc.context.compact_at_percent {
        cfg.context.compact_at_percent = v;
    }
}


fn merge_cli(cfg: &mut Config, o: CliOverrides) {
    if let Some(v) = o.base_url {
        cfg.llm.base_url = v;
    }
    if let Some(v) = o.model {
        cfg.llm.model = v;
    }
}

/// 写入口：仅支持 llm.* 三键；未知键报 CS1012。
pub fn apply_set(fc: &mut FileConfig, key: &str, value: String) -> CsResult<()> {
    let slot = match key {
        "llm.base_url" => &mut fc.llm.base_url,
        "llm.api_key" => &mut fc.llm.api_key,
        "llm.model" => &mut fc.llm.model,
        "vector.embed_model" => &mut fc.vector.embed_model,
        "vector.embed_mode" => &mut fc.vector.embed_mode,
        other => {
            return Err(CsError::new(CONFIG_INVALID, format!("未知配置键: {other}"))
                .with_hint("可用键: llm.base_url | llm.api_key | llm.model | vector.embed_model | vector.embed_mode（数值键 vector.embed_dims / vector.repomap_budget / behavior.thinking_on 走 config set-num）"));
        }
    };
    *slot = Some(value);
    Ok(())
}

/// 生效配置 → 文件视图（供 `config get` 展示）。
pub fn to_file_view(cfg: &Config) -> FileConfig {
    FileConfig {
        llm: FileLlm {
            base_url: Some(cfg.llm.base_url.clone()),
            api_key: cfg.llm.api_key.clone(),
            model: Some(cfg.llm.model.clone()),
        },
        context: FileContext::default(),
        vector: FileVector {
            embed_model: Some(cfg.vector.embed_model.clone()),
            embed_dims: Some(cfg.vector.embed_dims),
            embed_mode: Some(cfg.vector.embed_mode.clone()),
            repomap_budget: Some(cfg.vector.repomap_budget),
            base_url: cfg.vector.base_url.clone(),
            api_key: cfg.vector.api_key.clone(),
        },
        behavior: FileBehavior {
            thinking_on: Some(!cfg.thinking_disabled),
        },
    }
}

impl Config {
    /// 密钥解析：只认配置文件直配（env 层已移除；值不落日志）。
    pub fn resolve_api_key(&self) -> CsResult<String> {
        if let Some(k) = &self.llm.api_key
            && !k.trim().is_empty()
        {
            return Ok(k.clone());
        }
        Err({
            CsError::new(CONFIG_MISSING, "API key 未找到").with_hint(
                "在 ~/.codesleuth/config.toml 的 [llm] 段配置 api_key = \"...\" 后重试",
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_home_is_codesleuth_dir() {
        let p = global_config_path().unwrap();
        assert!(p.to_string_lossy().contains(".codesleuth"), "全局配置应在 ~/.codesleuth/ 下: {p:?}");
        assert!(p.ends_with("config.toml"));
        let d = global_state_dir().unwrap();
        assert!(d.to_string_lossy().contains(".codesleuth"));
    }

    fn write(path: &Path, content: &str) {
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn precedence_cli_beats_env_beats_project_beats_global() {
        let dir = tempfile::tempdir().unwrap();
        let g = dir.path().join("g.toml");
        let p = dir.path().join("p.toml");
        write(
            &g,
            "[llm]\nmodel = \"g-model\"\nbase_url = \"http://global\"\n",
        );
        write(
            &p,
            "[llm]\nmodel = \"p-model\"\nbase_url = \"http://project\"\n",
        );
        let cli = CliOverrides {
            model: Some("c-model".into()),
            ..Default::default()
        };
        let cfg = load_layered(Some(&g), Some(&p), cli).unwrap();
        // CLI > 项目 > 全局
        assert_eq!(cfg.llm.model, "c-model");
        assert_eq!(cfg.llm.base_url, "http://project");
    }

    #[test]
    fn api_key_direct_resolution() {
        let mut cfg = Config::default();
        cfg.llm.api_key = Some("from-file".into());
        assert_eq!(cfg.resolve_api_key().unwrap(), "from-file");
        cfg.llm.api_key = Some("   ".into());
        assert!(cfg.resolve_api_key().is_err(), "空白密钥视为未配置");
    }

    #[test]
    fn project_file_overrides_global_for_vector() {
        let dir = tempfile::tempdir().unwrap();
        let g = dir.path().join("g.toml");
        let pr = dir.path().join("p.toml");
        write(&g, "[vector]\nembed_model = \"global-model\"\nembed_dims = 512\n");
        write(&pr, "[vector]\nembed_model = \"project-model\"\n");
        let cfg = load_layered(Some(&g), Some(&pr), CliOverrides::default()).unwrap();
        assert_eq!(cfg.vector.embed_model, "project-model", "项目配置压全局");
        assert_eq!(cfg.vector.embed_dims, 512, "全局独有键保留");
    }

    #[test]
    fn thinking_defaults_off_and_file_can_enable() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("c.toml");
        write(&p, "[behavior]\nthinking_on = true\n");
        let cfg = load_layered(None, Some(&p), CliOverrides::default()).unwrap();
        assert!(!cfg.thinking_disabled, "配置文件 thinking_on = true 应开回思考");
    }

    #[test]
    fn missing_files_fall_back_to_defaults() {
        let cfg = load_layered(None, None, CliOverrides::default()).unwrap();
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn invalid_toml_is_cs1012() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("bad.toml");
        write(&p, "[llm\nmodel=");
        let err = load_layered(None, Some(&p), CliOverrides::default()).unwrap_err();
        assert_eq!(err.code, CONFIG_INVALID);
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn apply_set_rejects_unknown_key() {
        let mut fc = FileConfig::default();
        let err = apply_set(&mut fc, "llm.nope", "x".into()).unwrap_err();
        assert_eq!(err.code, CONFIG_INVALID);
        apply_set(&mut fc, "llm.model", "m".into()).unwrap();
        assert_eq!(fc.llm.model.as_deref(), Some("m"));
    }
}
