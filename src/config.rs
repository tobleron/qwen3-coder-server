use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuboxConfig {
    pub llm: LlmConfig,
    pub models: ModelsConfig,
    pub user: UserConfig,
    pub directories: DirectoriesConfig,
    pub cleanup: CleanupConfig,
    pub ui: UiConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LlmConfig {
    pub api_url: String,
    pub model_name: String,
    pub base_temp: f32,
    pub max_temp: f32,
    pub context_window: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelsConfig {
    pub default: String,
    pub registry: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserConfig {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirectoriesConfig {
    pub output: String,
    pub tmp_md: String,
    pub chat: String,
    pub prompts: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CleanupConfig {
    pub tmp_age_days: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiConfig {
    pub color_orange: String,
    pub color_red: String,
    pub color_dark_orange: String,
    pub color_bright_red: String,
    pub color_white: String,
    pub color_reset: String,
}

impl RuboxConfig {
    pub fn load() -> Self {
        let config_path = "rubox_config.json";
        if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path).expect("Failed to read config");
            serde_json::from_str(&content).unwrap_or_else(|_| Self::default_internal())
        } else {
            Self::default_internal()
        }
    }

    fn default_internal() -> Self {
        let mut registry = std::collections::HashMap::new();
        registry.insert(
            "qwen3-vl".to_string(),
            "models/Qwen3-VL-8B-Instruct-UD-Q6_K_XL.gguf".to_string(),
        );
        registry.insert(
            "gemma".to_string(),
            "models/google_gemma-3-4b-it-Q8_0.gguf".to_string(),
        );
        registry.insert(
            "lfm".to_string(),
            "models/LFM2.5-1.2B-Instruct-BF16.gguf".to_string(),
        );

        RuboxConfig {
            llm: LlmConfig {
                api_url: "http://127.0.0.1:8081/v1".to_string(),
                model_name: "qwen3-vl".to_string(),
                base_temp: 0.7,
                max_temp: 0.9,
                context_window: 8192,
            },
            models: ModelsConfig {
                default: "models/Qwen3-VL-8B-Instruct-UD-Q6_K_XL.gguf".to_string(),
                registry,
            },
            user: UserConfig {
                name: "Arto".to_string(),
            },
            directories: DirectoriesConfig {
                output: "output".to_string(),
                tmp_md: "tmp_md".to_string(),
                chat: "Chat".to_string(),
                prompts: "output/_prompts".to_string(),
            },
            cleanup: CleanupConfig { tmp_age_days: 3 },
            ui: UiConfig {
                color_orange: "\x1b[38;5;208m".to_string(),
                color_red: "\x1b[38;5;196m".to_string(),
                color_dark_orange: "\x1b[38;5;166m".to_string(),
                color_bright_red: "\x1b[38;5;9m".to_string(),
                color_white: "\x1b[37m".to_string(),
                color_reset: "\x1b[0m".to_string(),
            },
        }
    }
}
