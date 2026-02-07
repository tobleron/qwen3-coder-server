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

        let child = Command::new(server_path)
            .args(&[
                "--model", &model_path,
                "--ctx-size", &config.llm.context_window.to_string(),
                "--port", &port.to_string(),
                "--n-gpu-layers", "-1",
                "--parallel", "4",
                "--batch-size", "512",
                "--ubatch-size", "256",
                "--log-disable"
            ])
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

            if i > 5 && is_server_running(port).await {
                break;
            }

            tokio::time::sleep(Duration::from_millis(150)).await;
        }

        // Wait for actual readiness
        let start = std::time::Instant::now();
        loop {
            if is_server_running(port).await {
                let client = reqwest::Client::new();
                let res = client.get(format!("http://127.0.0.1:{}/health", port)).send().await;
                if let Ok(response) = res {
                    if response.status().is_success() {
                        print!("\r\x1b[K"); // Clear line
                        println!();
                        break;
                    }
                }
            }
            if start.elapsed().as_secs() > 120 {
                return Err(anyhow::anyhow!("Timeout waiting for llama-server to load model (120s)."));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
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

