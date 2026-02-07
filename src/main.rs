mod config;
mod llm_client;
mod server_manager;
mod chat;
mod multi_model;
mod ui;

use clap::Parser;
use std::fs;
use std::path::Path;
use config::RuboxConfig;
use llm_client::LlmClient;
use server_manager::ServerManager;

#[derive(Parser, Debug)]
#[command(name = "rubox")]
#[command(about = "Rust-based Chat Application with llama.cpp", long_about = None)]
struct Args {
    /// Model name or path override
    #[arg(short, long)]
    model: Option<String>,

    /// List available models from registry
    #[arg(short, long)]
    list: bool,

    /// Prompt text (if not using prompt_input.txt)
    #[arg(short, long)]
    prompt: Option<String>,

    /// Enable verbose mode
    #[arg(long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config
    let config = RuboxConfig::load();

    // Parse CLI args
    let args = Args::parse();

    // Handle --list
    if args.list {
        println!(
            "{}Available models:{}",
            config.ui.color_orange, config.ui.color_reset
        );
        for (name, path) in &config.models.registry {
            println!("  {} -> {}", name, path);
        }
        return Ok(());
    }

    // Disable Ollama from systemd (user may have sudo access to run this)
    // The app will not attempt to stop ollama automatically
    if args.verbose {
        println!("{}Note: Ollama service will not be stopped. To disable it from systemd, run: sudo systemctl disable ollama{}\n",
            config.ui.color_orange, config.ui.color_reset);
    }

    // Create directories
    ensure_directories(&config)?;

    // Get prompt - either from CLI, prompt_input.txt, or ask user
    let prompt = get_or_prompt_user(&config)?;

    // Get available models
    let mut available_models = get_available_models(&config);
    available_models.sort();

    // Display available models and get selection
    ui::display_model_list(&available_models, &config);
    let selection = ui::read_model_selection(&available_models)?;

    // Remove duplicates and sort selection
    let mut unique_selection = selection;
    unique_selection.sort();
    unique_selection.dedup();

    // Branch: Chat mode vs Multi-model mode
    if unique_selection.len() == 1 {
        // Chat mode - single model
        let selected_model = &available_models[unique_selection[0]];
        let mut server = ServerManager::new();
        server.ensure_running(&config, Some(selected_model)).await?;

        let client = LlmClient::new(&config);
        chat::run_chat_mode(&client, selected_model, prompt, &config, args.verbose, &mut server).await?;
    } else {
        // Multi-model mode
        let selected_model_names: Vec<_> = unique_selection
            .iter()
            .map(|&i| available_models[i].clone())
            .collect();

        let mut server = ServerManager::new();
        multi_model::run_multi_model(selected_model_names, prompt, &config, &mut server).await?;
    }

    // Cleanup
    cleanup_old_files(&config)?;
    clear_prompt_file("prompt_input.txt")?;

    Ok(())
}

fn ensure_directories(config: &RuboxConfig) -> anyhow::Result<()> {
    fs::create_dir_all(&config.directories.output)?;
    fs::create_dir_all(&config.directories.tmp_md)?;
    fs::create_dir_all(&config.directories.chat)?;
    fs::create_dir_all(&config.directories.prompts)?;
    Ok(())
}

fn get_or_prompt_user(config: &RuboxConfig) -> anyhow::Result<String> {
    let prompt_file = "prompt_input.txt";

    // Check if prompt_input.txt exists and has content
    if let Ok(content) = fs::read_to_string(prompt_file) {
        if !content.trim().is_empty() {
            println!();
            println!(
                "{}✓ Existing prompt detected in prompt_input.txt. Proceeding...{}",
                config.ui.color_orange, config.ui.color_reset
            );
            println!();
            return Ok(content.trim().to_string());
        }
    }

    // Ask user for prompt
    println!();
    println!(
        "{}═══════════════════════════════════════{}",
        config.ui.color_orange, config.ui.color_reset
    );
    println!(
        "{}Enter your query {}:{}",
        config.ui.color_orange, config.user.name, config.ui.color_reset
    );
    println!(
        "{}═══════════════════════════════════════{}",
        config.ui.color_orange, config.ui.color_reset
    );
    let mut prompt = String::new();
    std::io::stdin().read_line(&mut prompt)?;
    println!();

    if prompt.trim().is_empty() {
        anyhow::bail!("Invalid input. Exiting...");
    }

    Ok(prompt.trim().to_string())
}

fn get_available_models(config: &RuboxConfig) -> Vec<String> {
    let mut models = Vec::new();

    // Add models from registry
    for name in config.models.registry.keys() {
        models.push(name.clone());
    }

    // Add models from models directory
    if let Ok(entries) = fs::read_dir(&Path::new("models")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let model_name = file_name.replace(".gguf", "");
                    if !models.contains(&model_name) {
                        models.push(model_name);
                    }
                }
            }
        }
    }

    models
}

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

fn clear_prompt_file(path: &str) -> anyhow::Result<()> {
    if Path::new(path).exists() {
        fs::write(path, "")?;
    }
    Ok(())
}
