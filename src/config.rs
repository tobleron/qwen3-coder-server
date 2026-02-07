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
    pub temperature: TemperatureConfig,
    pub session: SessionConfig,
    #[serde(default = "ModelProfiles::default_profiles")]
    pub model_profiles: std::collections::HashMap<String, ModelParams>,
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
    pub sessions: String,
    pub static_prompts: String,
    pub saved_responses: String,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemperatureConfig {
    pub default: f32,
    pub min: f32,
    pub max: f32,
    pub allow_override: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionConfig {
    pub auto_save: bool,
    pub format: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelParams {
    pub batch_size: u32,
    pub ubatch_size: u32,
    pub gpu_layers: i32,
    pub context_window: u32,
    pub mmproj: Option<String>, // Vision model projection file
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelProfiles {
    pub profiles: std::collections::HashMap<String, ModelParams>,
}

impl ModelProfiles {
    fn default_profiles() -> std::collections::HashMap<String, ModelParams> {
        let mut profiles = std::collections::HashMap::new();

        // Qwen3-VL-8B: Optimized for RTX 3060 12GB with vision support
        profiles.insert(
            "qwen3-vl".to_string(),
            ModelParams {
                batch_size: 512,
                ubatch_size: 256,
                gpu_layers: 35,
                context_window: 8192,
                mmproj: Some("models/mmproj-Qwen3VL-8B-Instruct-F16.gguf".to_string()),
            },
        );

        // Gemma 3 4B: Lightweight text model
        profiles.insert(
            "gemma".to_string(),
            ModelParams {
                batch_size: 1024,
                ubatch_size: 512,
                gpu_layers: 45,
                context_window: 8192,
                mmproj: None,
            },
        );

        // LFM 1.2B: Ultra-lightweight model
        profiles.insert(
            "lfm".to_string(),
            ModelParams {
                batch_size: 1024,
                ubatch_size: 512,
                gpu_layers: 99,
                context_window: 4096,
                mmproj: None,
            },
        );

        // Qwen3-128k-30B: Extended context reasoning model (IQ2_M 10GB)
        // MOE model with 8 experts active by default
        // Optimal for reasoning, deep thinking tasks
        profiles.insert(
            "qwen3-128k".to_string(),
            ModelParams {
                batch_size: 256,      // Conservative for 30B model on 12GB
                ubatch_size: 128,     // Micro-batch size
                gpu_layers: 60,       // Push most layers to GPU (30B model)
                context_window: 32768, // 32k tokens (safe), can extend to 128k if needed
                mmproj: None,         // Text-only model
            },
        );

        profiles
    }
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

    pub fn get_model_params(&self, model_name: &str) -> ModelParams {
        self.model_profiles
            .get(model_name)
            .cloned()
            .unwrap_or_else(|| ModelParams {
                batch_size: 512,
                ubatch_size: 256,
                gpu_layers: 35,
                context_window: 8192,
                mmproj: None,
            })
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
        registry.insert(
            "qwen3-128k".to_string(),
            "models/Qwen3-128k-30B-NEO-MAX-PLUS-IQ2_M.gguf".to_string(),
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
                sessions: "Chat/sessions".to_string(),
                static_prompts: "prompts/static".to_string(),
                saved_responses: "Chat/saved".to_string(),
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
            temperature: TemperatureConfig {
                default: 0.7,
                min: 0.0,
                max: 2.0,
                allow_override: true,
            },
            session: SessionConfig {
                auto_save: true,
                format: "json".to_string(),
            },
            model_profiles: ModelProfiles::default_profiles(),
        }
    }
}
