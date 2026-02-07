use std::process::{Command, Child, Stdio};
use std::time::Duration;
use std::net::TcpStream;
use std::io::{self, Write};
use crate::config::RuboxConfig;

pub struct ServerManager {
    child: Option<Child>,
}

impl ServerManager {
    pub fn new() -> Self {
        ServerManager { child: None }
    }

    pub async fn ensure_running(&mut self, config: &RuboxConfig, model_override: Option<&str>) -> anyhow::Result<()> {
        // Parse port from API URL
        let port = config.llm.api_url
            .split(':')
            .last()
            .and_then(|s| s.split('/').next())
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(8081);

        // Check if something is already listening
        if is_server_running(port).await {
            return Ok(());
        }

        println!();
        println!("{}═══════════════════════════════════════{}", config.ui.color_orange, config.ui.color_reset);
        println!("{}   Initializing LLM Engine...{}", config.ui.color_orange, config.ui.color_reset);
        println!("{}═══════════════════════════════════════{}", config.ui.color_orange, config.ui.color_reset);

        // Determine model path
        let model_path = if let Some(path) = model_override {
            if path.contains('/') || path.contains('.') {
                // Direct file path
                path.to_string()
            } else {
                // Look up in registry
                config.models.registry
                    .get(path)
                    .cloned()
                    .unwrap_or_else(|| config.models.default.clone())
            }
        } else {
            config.models.default.clone()
        };

        let server_path = "./third_party/llama.cpp/build/bin/llama-server";

        // Get model-specific parameters
        let model_key = if let Some(override_name) = model_override {
            if !override_name.contains('/') && !override_name.contains('.') {
                override_name.to_string()
            } else {
                config.models.registry
                    .iter()
                    .find(|(_, path)| path.contains(override_name))
                    .map(|(name, _)| name.clone())
                    .unwrap_or_else(|| "qwen3-vl".to_string())
            }
        } else {
            "qwen3-vl".to_string()
        };

        let model_params = config.get_model_params(&model_key);

        // Build command with model-specific parameters
        let mut cmd = Command::new(server_path);
        cmd.args(&[
            "--model", &model_path,
            "--ctx-size", &model_params.context_window.to_string(),
            "--port", &port.to_string(),
            "--n-gpu-layers", &model_params.gpu_layers.to_string(),
            "--parallel", "4",
            "--batch-size", &model_params.batch_size.to_string(),
            "--ubatch-size", &model_params.ubatch_size.to_string(),
            "--log-disable"
        ]);

        // Add vision model projection if present
        if let Some(mmproj_path) = &model_params.mmproj {
            if std::path::Path::new(mmproj_path).exists() {
                cmd.args(&["--mmproj", mmproj_path]);
            }
        }

        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // Progress simulation
        let states = ["Loading Config", "Allocating Context", "Loading Model Weights", "Warming Up"];
        let total_steps = 20;

        for i in 0..total_steps {
            let percentage = (i as f32 / total_steps as f32 * 100.0) as u32;
            let filled = (i as f32 / total_steps as f32 * 20.0) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(20_usize.saturating_sub(filled));

            let state_idx = (i as usize * states.len()) / total_steps;
            let current_state = states[state_idx.min(states.len() - 1)];

            print!("\r   {}[{}] {}% - {}...{}", config.ui.color_orange, bar, percentage, current_state, config.ui.color_reset);
            let _ = io::stdout().flush();

            tokio::time::sleep(Duration::from_millis(250)).await;
        }

        // Wait for server to be ready
        print!("\r\x1b[K"); // Clear line
        let start = std::time::Instant::now();
        let max_wait = 180; // 3 minutes for large models

        loop {
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Check if server is responding on port
            if is_server_running(port).await {
                // Try a simple health check with generous timeout
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()?;

                match client.get(format!("http://127.0.0.1:{}/health", port)).send().await {
                    Ok(_) => {
                        // Server responded - wait longer for large models to fully initialize
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        print!("\r\x1b[K");
                        println!();
                        break;
                    }
                    Err(_) => {
                        // Health check failed, but server is listening - might still be loading
                        // Continue waiting
                    }
                }
            }

            // Show progress dots
            let elapsed = start.elapsed().as_secs();
            if elapsed % 10 == 0 {
                print!(".");
                let _ = io::stdout().flush();
            }

            if elapsed > max_wait {
                print!("\r\x1b[K");
                return Err(anyhow::anyhow!("Timeout waiting for llama-server ({}s).", max_wait));
            }
        }

        self.child = Some(child);
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
        Ok(())
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}

async fn is_server_running(port: u16) -> bool {
    match TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_millis(200),
    ) {
        Ok(_) => true,
        Err(_) => false,
    }
}

