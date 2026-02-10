use std::fs;
use std::path::Path;
use std::time::Instant;
use chrono::Local;
use crate::config::RuboxConfig;
use crate::llm_client::{LlmClient, ChatMessage};
use crate::server_manager::ServerManager;

#[allow(dead_code)]
pub async fn run_multi_model(
    selected_models: Vec<String>,
    prompt: String,
    config: &RuboxConfig,
    server_manager: &mut ServerManager,
) -> anyhow::Result<()> {
    let timestamp = Local::now().format("%d_%m_%Y_%H_%M_%S").to_string();

    // Create directories
    fs::create_dir_all(&config.directories.prompts)?;
    fs::create_dir_all(&config.directories.tmp_md)?;
    fs::create_dir_all(&config.directories.output)?;

    // Save prompt
    let prompt_file = format!("{}/Prompt_{}.md", config.directories.prompts, timestamp);
    fs::write(&prompt_file, &prompt)?;

    let mut results = String::new();

    for model_name in selected_models {
        // Stop and restart server with new model
        server_manager.stop()?;
        server_manager.ensure_running(config, Some(&model_name)).await?;

        // Create client and send prompt
        let client = LlmClient::new(config);
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.clone(),
        }];

        let start_time = Instant::now();
        match client.chat_completion_with_usage(messages, config.temperature.default).await {
            Ok((response, usage)) => {
                let elapsed = start_time.elapsed();

                // Save individual response
                let sanitized_model = sanitize_filename(&model_name);
                let response_file = format!(
                    "{}/{}_{}.md",
                    config.directories.tmp_md, sanitized_model, timestamp
                );
                fs::write(&response_file, &response)?;

                // Display response
                println!();
                println!("{}┌─ {} {}{}", config.ui.color_orange, model_name, "─", config.ui.color_reset);
                println!("{}{}{}", config.ui.color_orange, response, config.ui.color_reset);

                let tps = if let Some(usage) = &usage {
                    let tokens = usage.completion_tokens as f32;
                    tokens / elapsed.as_secs_f32()
                } else {
                    0.0
                };
                println!(
                    "{}└─ ({:.1} tps, {:.2}s){} {}",
                    config.ui.color_orange, tps, elapsed.as_secs_f32(), "─", config.ui.color_reset
                );
                println!();

                // Add to results
                if !results.is_empty() {
                    results.push_str("\n---\n\n");
                }
                results.push_str(&format!("# {}\n\n{}\n", model_name, response));
            }
            Err(e) => {
                eprintln!(
                    "{}⚠ Error getting response from {}: {}{}",
                    config.ui.color_orange, model_name, e, config.ui.color_reset
                );
            }
        }
    }

    // Save combined results
    let results_file = format!("{}/Results_{}.md", config.directories.output, timestamp);
    fs::write(&results_file, &results)?;
    println!();
    println!(
        "{}═══════════════════════════════════════{}",
        config.ui.color_orange, config.ui.color_reset
    );
    println!(
        "{}Results saved to: {}{}",
        config.ui.color_orange, results_file, config.ui.color_reset
    );
    println!(
        "{}═══════════════════════════════════════{}",
        config.ui.color_orange, config.ui.color_reset
    );
    println!();

    // Cleanup old files
    cleanup_old_files(config)?;

    Ok(())
}

#[allow(dead_code)]
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            ':' | '\\' | '/' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

#[allow(dead_code)]
fn cleanup_old_files(config: &RuboxConfig) -> anyhow::Result<()> {
    let tmp_path = Path::new(&config.directories.tmp_md);
    if !tmp_path.exists() {
        return Ok(());
    }

    let cutoff_time = std::time::SystemTime::now()
        - std::time::Duration::from_secs((config.cleanup.tmp_age_days as u64) * 86400);

    for entry in fs::read_dir(tmp_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff_time {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }

    Ok(())
}
