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
    #[serde(default)]
    pub graph: FileGraph,
}

/// 结图层文件形态（P005 R7.2：CODEGRAPH_BIN env 收编进配置链）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileGraph {
    /// codegraph 可执行名（缺席 = "codegraph"）。
    pub bin: Option<String>,
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
    /// 多模型档位（D017）：[llm.profiles.<名字>]，CLI --profile 选用；缺席字段跟随主 [llm]。
    #[serde(default)]
    pub profiles: std::collections::BTreeMap<String, FileProfile>,
}

/// 模型档位（D017）：连接面全套可选覆盖，缺席字段跟随主 [llm]。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileProfile {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub model_context_tokens: Option<u64>,
}

/// 运行时生效配置。
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub llm: LlmConfig,
    pub context: ContextConfig,
    pub vector: VectorConfig,
    /// 结图层（P005 R7.2）。
    pub graph: GraphConfig,
    pub thinking_disabled: bool,
    /// 生效模型档位（D017）：None = 未用 --profile，走主 [llm]。
    pub active_profile: Option<String>,
}

/// 结图层参数（P005 R7.2）。
#[derive(Debug, Clone, PartialEq)]
pub struct GraphConfig {
    pub bin: String,
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
            graph: GraphConfig {
                bin: "codegraph".into(),
            },
            active_profile: None,
        }
    }
}

/// CLI 覆盖（与 clap 解耦）。
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub base_url: Option<String>,
    pub model: Option<String>,
    /// 模型档位名（D017）：解析 [llm.profiles.<名字>]，旗标仍可再压。
    pub profile: Option<String>,
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
    let mut profiles: std::collections::BTreeMap<String, FileProfile> = Default::default();
    for path in [global, project].into_iter().flatten() {
        let fc = parse_file(path)?;
        // D017：档位表逐层收集，项目层同名覆盖全局；连接面本身照旧 merge
        profiles.extend(fc.llm.profiles.clone());
        merge_file(&mut cfg, fc);
    }
    apply_profile(&mut cfg, &profiles, cli.profile.as_deref())?;
    merge_cli(&mut cfg, cli);
    Ok(cfg)
}

/// D017 档位解析：存在性校验（未知名报错并列出可用档位）→ 逐字段覆盖（缺席跟随上层）。
/// 先于 merge_cli 执行——--model / --base-url 旗标永远是最后覆写者。
fn apply_profile(
    cfg: &mut Config,
    profiles: &std::collections::BTreeMap<String, FileProfile>,
    name: Option<&str>,
) -> CsResult<()> {
    let Some(name) = name else {
        return Ok(());
    };
    let prof = profiles.get(name).ok_or_else(|| {
        let names = if profiles.is_empty() {
            "（配置文件未定义任何档位）".to_string()
        } else {
            profiles.keys().cloned().collect::<Vec<_>>().join(", ")
        };
        CsError::new(CONFIG_INVALID, format!("未知模型档位: --profile {name}")).with_hint(format!(
            "可用档位: {names}（经 [llm.profiles.<名字>] 定义于配置文件）"
        ))
    })?;
    if let Some(v) = &prof.base_url {
        cfg.llm.base_url = v.clone();
    }
    if let Some(v) = &prof.api_key {
        cfg.llm.api_key = Some(v.clone());
    }
    if let Some(v) = &prof.model {
        cfg.llm.model = v.clone();
    }
    if let Some(v) = prof.model_context_tokens {
        cfg.context.model_context_tokens = v;
    }
    cfg.active_profile = Some(name.to_string());
    Ok(())
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
    if let Some(v) = fc.graph.bin {
        cfg.graph.bin = v;
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

/// 配置键表（P005 R7.2 表驱动）：一处声明，apply_set / resolved_get / 未知键提示三处共用。
/// 加新键 = 在 `config_key_table()` 加一行（此前要改 4-5 处，coupling 审查域4）。
pub struct ConfigKey {
    pub name: &'static str,
    /// 生效配置 → 展示值。
    pub get: fn(&Config) -> String,
    /// 文件视图写入（含类型校验）。
    pub set: fn(&mut FileConfig, String) -> CsResult<()>,
}

fn parse_typed<T: std::str::FromStr>(key: &str, value: &str) -> CsResult<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    value
        .parse::<T>()
        .map_err(|e| CsError::new(CONFIG_INVALID, format!("{key} 值非法: {value}（{e}）")))
}

pub fn config_key_table() -> &'static [ConfigKey] {
    &[
        ConfigKey {
            name: "llm.base_url",
            get: |c| c.llm.base_url.clone(),
            set: |fc, v| {
                fc.llm.base_url = Some(v);
                Ok(())
            },
        },
        ConfigKey {
            name: "llm.api_key",
            get: |c| c.llm.api_key.clone().unwrap_or_default(),
            set: |fc, v| {
                fc.llm.api_key = Some(v);
                Ok(())
            },
        },
        ConfigKey {
            name: "llm.model",
            get: |c| c.llm.model.clone(),
            set: |fc, v| {
                fc.llm.model = Some(v);
                Ok(())
            },
        },
        ConfigKey {
            name: "llm.profile",
            // D017：只读键——档位经 [llm.profiles.<名字>] 定义，运行时 --profile 选用
            get: |c| c.active_profile.clone().unwrap_or_else(|| "default".into()),
            set: |_fc, _v| {
                Err(CsError::new(
                    CONFIG_INVALID,
                    "llm.profile 只读：档位经 [llm.profiles.<名字>] 定义，运行时用 --profile 选用",
                ))
            },
        },
        ConfigKey {
            name: "context.model_context_tokens",
            get: |c| c.context.model_context_tokens.to_string(),
            set: |fc, v| {
                fc.context.model_context_tokens =
                    Some(parse_typed("context.model_context_tokens", &v)?);
                Ok(())
            },
        },
        ConfigKey {
            name: "context.compact_at_percent",
            get: |c| c.context.compact_at_percent.to_string(),
            set: |fc, v| {
                fc.context.compact_at_percent =
                    Some(parse_typed("context.compact_at_percent", &v)?);
                Ok(())
            },
        },
        ConfigKey {
            name: "vector.embed_model",
            get: |c| c.vector.embed_model.clone(),
            set: |fc, v| {
                fc.vector.embed_model = Some(v);
                Ok(())
            },
        },
        ConfigKey {
            name: "vector.embed_dims",
            get: |c| c.vector.embed_dims.to_string(),
            set: |fc, v| {
                fc.vector.embed_dims = Some(parse_typed("vector.embed_dims", &v)?);
                Ok(())
            },
        },
        ConfigKey {
            name: "vector.embed_mode",
            get: |c| c.vector.embed_mode.clone(),
            set: |fc, v| {
                fc.vector.embed_mode = Some(v);
                Ok(())
            },
        },
        ConfigKey {
            name: "vector.base_url",
            // 缺省跟随 [llm].base_url（嵌入专用端点语义，P005 R7.1 同口径）
            get: |c| {
                c.vector
                    .base_url
                    .clone()
                    .unwrap_or_else(|| c.llm.base_url.clone())
            },
            set: |fc, v| {
                fc.vector.base_url = Some(v);
                Ok(())
            },
        },
        ConfigKey {
            name: "vector.api_key",
            get: |c| {
                c.vector
                    .api_key
                    .clone()
                    .unwrap_or_else(|| c.llm.api_key.clone().unwrap_or_default())
            },
            set: |fc, v| {
                fc.vector.api_key = Some(v);
                Ok(())
            },
        },
        ConfigKey {
            name: "vector.repomap_budget",
            get: |c| c.vector.repomap_budget.to_string(),
            set: |fc, v| {
                fc.vector.repomap_budget = Some(parse_typed("vector.repomap_budget", &v)?);
                Ok(())
            },
        },
        ConfigKey {
            name: "behavior.thinking_on",
            get: |c| (!c.thinking_disabled).to_string(),
            set: |fc, v| {
                let b = v.parse::<bool>().map_err(|_| {
                    CsError::new(CONFIG_INVALID, format!("thinking_on 需要布尔: {v}"))
                        .with_hint("取值: true | false")
                })?;
                fc.behavior.thinking_on = Some(b);
                Ok(())
            },
        },
        ConfigKey {
            name: "graph.bin",
            get: |c| c.graph.bin.clone(),
            set: |fc, v| {
                fc.graph.bin = Some(v);
                Ok(())
            },
        },
    ]
}

/// 键表全名提示（未知键报错的 hint 用）。
pub fn keys_hint() -> String {
    config_key_table()
        .iter()
        .map(|k| k.name)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn find_key(key: &str) -> CsResult<&'static ConfigKey> {
    config_key_table()
        .iter()
        .find(|k| k.name == key)
        .ok_or_else(|| {
            CsError::new(CONFIG_INVALID, format!("未知配置键: {key}"))
                .with_hint(format!("可用键: {}", keys_hint()))
        })
}

/// 写入口：键表驱动（P005 R7.2）；未知键报 CS1012。
pub fn apply_set(fc: &mut FileConfig, key: &str, value: String) -> CsResult<()> {
    let k = find_key(key)?;
    (k.set)(fc, value)
}

/// 读入口：生效配置按键取展示值（P005 R7.2，与 apply_set 同表）。
pub fn resolved_get(cfg: &Config, key: &str) -> CsResult<String> {
    let k = find_key(key)?;
    Ok((k.get)(cfg))
}

/// 生效配置 → 文件视图（供 `config get` 展示）。
pub fn to_file_view(cfg: &Config) -> FileConfig {
    FileConfig {
        llm: FileLlm {
            base_url: Some(cfg.llm.base_url.clone()),
            api_key: cfg.llm.api_key.clone(),
            model: Some(cfg.llm.model.clone()),
            profiles: Default::default(),
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
        graph: FileGraph {
            bin: Some(cfg.graph.bin.clone()),
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
            CsError::new(CONFIG_MISSING, "API key 未找到")
                .with_hint("在 ~/.codesleuth/config.toml 的 [llm] 段配置 api_key = \"...\" 后重试")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P005 R7.2：键表全键 set → 写盘 → load → get 闭环 + 未知键报错。
    #[test]
    fn key_table_roundtrip_and_unknown_key() {
        let dir = tempfile::tempdir().unwrap();
        let g = dir.path().join("g.toml");
        let mut fc = FileConfig::default();
        // D017 只读键：set 必须拒绝，不走 set→get 往返（get 期望缺省值）
        const READ_ONLY_KEYS: &[&str] = &["llm.profile"];
        for k in config_key_table() {
            if READ_ONLY_KEYS.contains(&k.name) {
                assert!(
                    (k.set)(&mut fc, "x".into()).is_err(),
                    "只读键 {} 的 set 应拒绝",
                    k.name
                );
                continue;
            }
            (k.set)(&mut fc, sample_value(k.name).to_string()).unwrap();
        }
        std::fs::write(&g, toml::to_string_pretty(&fc).unwrap()).unwrap();
        let cfg = load_layered(Some(&g), None, CliOverrides::default()).unwrap();
        for k in config_key_table() {
            let got = resolved_get(&cfg, k.name).unwrap();
            let want = if READ_ONLY_KEYS.contains(&k.name) {
                "default"
            } else {
                expected_value(k.name)
            };
            assert_eq!(got, want, "键 {} 回读不符", k.name);
        }
        // 未知键：set/get 都报 CS1012 且 hint 含全部键名（含新增 context.*/graph.bin）
        let err = apply_set(&mut fc, "nope.key", "1".into()).unwrap_err();
        assert_eq!(err.code, CONFIG_INVALID);
        assert!(err.hint.as_deref().unwrap().contains("graph.bin"));
        assert!(resolved_get(&cfg, "nope.key").is_err());
    }

    fn sample_value(name: &str) -> &'static str {
        match name {
            "llm.base_url" => "http://set-llm",
            "llm.api_key" => "sk-set",
            "llm.model" => "set-model",
            "context.model_context_tokens" => "500000",
            "context.compact_at_percent" => "50",
            "vector.embed_model" => "set-embed",
            "vector.embed_dims" => "512",
            "vector.embed_mode" => "raw",
            "vector.base_url" => "http://set-vec",
            "vector.api_key" => "sk-vec",
            "vector.repomap_budget" => "1000",
            "behavior.thinking_on" => "false",
            "graph.bin" => "cg-set",
            _ => unreachable!(),
        }
    }

    fn expected_value(name: &str) -> &'static str {
        // behavior.thinking_on 经 thinking_disabled 取反往返后与原值一致（set false → get false）
        sample_value(name)
    }

    #[test]
    fn global_home_is_codesleuth_dir() {
        let p = global_config_path().unwrap();
        assert!(
            p.to_string_lossy().contains(".codesleuth"),
            "全局配置应在 ~/.codesleuth/ 下: {p:?}"
        );
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
        write(
            &g,
            "[vector]\nembed_model = \"global-model\"\nembed_dims = 512\n",
        );
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
        assert!(
            !cfg.thinking_disabled,
            "配置文件 thinking_on = true 应开回思考"
        );
    }

    #[test]
    fn missing_files_fall_back_to_defaults() {
        let cfg = load_layered(None, None, CliOverrides::default()).unwrap();
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn profile_overrides_connection_and_window() {
        // D017：档位覆盖连接面全套 + 窗口；缺席字段跟随主 [llm]
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("c.toml");
        write(
            &p,
            "[llm]\nmodel = \"main-model\"\nbase_url = \"http://main\"\napi_key = \"sk-main\"\n\
             [llm.profiles.scout]\nmodel = \"scout-model\"\nmodel_context_tokens = 131072\n",
        );
        let cli = CliOverrides {
            profile: Some("scout".into()),
            ..Default::default()
        };
        let cfg = load_layered(None, Some(&p), cli).unwrap();
        assert_eq!(cfg.llm.model, "scout-model", "档位 model 覆盖");
        assert_eq!(
            cfg.llm.base_url, "http://main",
            "缺席 base_url 跟随主 [llm]"
        );
        assert_eq!(
            cfg.llm.api_key.as_deref(),
            Some("sk-main"),
            "缺席 api_key 跟随"
        );
        assert_eq!(cfg.context.model_context_tokens, 131072, "窗口随档位");
        assert_eq!(cfg.active_profile.as_deref(), Some("scout"));
    }

    #[test]
    fn profile_full_override_and_cli_still_wins() {
        // D017：档位全套覆盖可用；--model/--base-url 旗标永远最后覆写
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("c.toml");
        write(
            &p,
            "[llm]\nmodel = \"main-model\"\nbase_url = \"http://main\"\n\
             [llm.profiles.review]\nmodel = \"review-model\"\nbase_url = \"http://review\"\napi_key = \"sk-review\"\n",
        );
        let cli = CliOverrides {
            profile: Some("review".into()),
            model: Some("flag-model".into()),
            ..Default::default()
        };
        let cfg = load_layered(None, Some(&p), cli).unwrap();
        assert_eq!(cfg.llm.model, "flag-model", "旗标 > 档位");
        assert_eq!(cfg.llm.base_url, "http://review", "旗标缺席字段用档位值");
        assert_eq!(cfg.llm.api_key.as_deref(), Some("sk-review"));
    }

    #[test]
    fn unknown_profile_lists_available_names() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("c.toml");
        write(
            &p,
            "[llm.profiles.scout]\nmodel = \"a\"\n[llm.profiles.review]\nmodel = \"b\"\n",
        );
        let cli = CliOverrides {
            profile: Some("nope".into()),
            ..Default::default()
        };
        let err = load_layered(None, Some(&p), cli).unwrap_err();
        assert_eq!(err.code, CONFIG_INVALID);
        let hint = err.hint.unwrap_or_default();
        assert!(
            hint.contains("scout") && hint.contains("review"),
            "应列出可用档位: {hint}"
        );
    }

    #[test]
    fn project_profile_overrides_global_same_name() {
        // D017：项目层同名档位压全局
        let dir = tempfile::tempdir().unwrap();
        let g = dir.path().join("g.toml");
        let pr = dir.path().join("p.toml");
        write(&g, "[llm.profiles.scout]\nmodel = \"global-scout\"\n");
        write(&pr, "[llm.profiles.scout]\nmodel = \"project-scout\"\n");
        let cli = CliOverrides {
            profile: Some("scout".into()),
            ..Default::default()
        };
        let cfg = load_layered(Some(&g), Some(&pr), cli).unwrap();
        assert_eq!(cfg.llm.model, "project-scout", "项目同名档位压全局");
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
