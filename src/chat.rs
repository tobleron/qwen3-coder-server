use std::io::{self, Write};
use chrono::Local;
use std::fs;
use std::path::Path;
use std::time::Instant;
use colored::*;
use crate::config::RuboxConfig;
use crate::llm_client::{LlmClient, ChatMessage};
use crate::server_manager::ServerManager;

enum CommandResult {
    Continue,
    Exit,
}

pub async fn run_chat_mode(
    client: &LlmClient,
    model_name: &str,
    initial_prompt: String,
    config: &RuboxConfig,
    mut verbose: bool,
    server: &mut ServerManager,
) -> anyhow::Result<()> {
    let mut history = vec![ChatMessage {
        role: "user".to_string(),
        content: initial_prompt.clone(),
    }];

    let mut current_model = model_name.to_string();

    println!();
    print_welcome_panel(config);
    println!();

    let mut chat_active = true;

    while chat_active {
        // Send entire history to LLM
        let start_time = Instant::now();
        let (response, usage) = client.chat_completion_with_usage(history.clone()).await?;
        let elapsed = start_time.elapsed();

        // Display model name and response
        println!();
        print_turn_header(&current_model, config);
        print_response_panel(&response, config);

        // Show timing info for verbose mode
        if verbose {
            let tps = if let Some(usage) = &usage {
                let tokens = usage.completion_tokens as f32;
                tokens / elapsed.as_secs_f32()
            } else {
                0.0
            };
            print_stats_panel(tps, elapsed.as_secs_f32(), config);
        }
        println!();

        // Append assistant response to history
        history.push(ChatMessage {
            role: "assistant".to_string(),
            content: response,
        });

        // Prompt for user input
        print_user_prompt(config)?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();
        println!();

        if user_input.starts_with('/') {
            match handle_slash_command(
                user_input,
                &mut current_model,
                &mut verbose,
                server,
                config,
                &history,
                client,
            )
            .await? {
                CommandResult::Exit => {
                    chat_active = false;
                }
                CommandResult::Continue => {
                    // Continue loop without adding to history
                }
            }
        } else if !user_input.is_empty() {
            // Append user message to history
            history.push(ChatMessage {
                role: "user".to_string(),
                content: user_input.to_string(),
            });
        }
    }

    Ok(())
}

async fn handle_slash_command(
    input: &str,
    current_model: &mut String,
    verbose: &mut bool,
    server: &mut ServerManager,
    config: &RuboxConfig,
    history: &[ChatMessage],
    client: &LlmClient,
) -> anyhow::Result<CommandResult> {
    let parts: Vec<&str> = input[1..].split_whitespace().collect();
    let command = parts.first().unwrap_or(&"");

    match *command {
        "" => {
            // Just "/" - show available commands
            show_help(config);
            Ok(CommandResult::Continue)
        }
        "exit" | "quit" => {
            save_chat(history, current_model, config)?;
            Ok(CommandResult::Exit)
        }
        "verbose" => {
            *verbose = !*verbose;
            let status = if *verbose {
                "✓ Verbose mode enabled".bright_green()
            } else {
                "✗ Verbose mode disabled".bright_red()
            };
            println!("{}  {}", "  ", status);
            println!();
            Ok(CommandResult::Continue)
        }
        "model" => {
            handle_model_command(&parts[1..], current_model, server, config, client).await?;
            Ok(CommandResult::Continue)
        }
        _ => {
            println!(
                "{}✗ Unknown command: /{}. Type / to see available commands.{}",
                "  ".bright_red(), command, config.ui.color_reset
            );
            println!();
            Ok(CommandResult::Continue)
        }
    }
}

fn show_help(config: &RuboxConfig) {
    println!();
    print_panel_header("Available Commands", config);
    println!(
        "{}  /exit       {}Save chat and exit{}",
        "  ".bright_green(),
        "─ ".bright_green(),
        config.ui.color_reset
    );
    println!(
        "{}  /verbose    {}Toggle timing info{}",
        "  ".bright_green(),
        "─ ".bright_green(),
        config.ui.color_reset
    );
    println!(
        "{}  /model      {}List available models{}",
        "  ".bright_green(),
        "─ ".bright_green(),
        config.ui.color_reset
    );
    println!(
        "{}  /model <N>  {}Switch to model N{}",
        "  ".bright_green(),
        "─ ".bright_green(),
        config.ui.color_reset
    );
    print_panel_footer(config);
    println!();
}

async fn handle_model_command(
    args: &[&str],
    current_model: &mut String,
    server: &mut ServerManager,
    config: &RuboxConfig,
    _client: &LlmClient,
) -> anyhow::Result<()> {
    let mut models = get_available_models_list(config);
    models.sort();

    if args.is_empty() {
        // List models
        println!();
        print_panel_header("Available Models", config);
        for (i, model) in models.iter().enumerate() {
            let current_marker = if model == current_model {
                format!(" {}", "✓ (current)".bright_green())
            } else {
                String::new()
            };
            println!(
                "{}  [{}] {}{}{}",
                "  ".bright_green(),
                (i + 1).to_string().bright_green(),
                model.bright_green(),
                current_marker,
                config.ui.color_reset
            );
        }
        println!(
            "{}  Use {}{}{}{}",
            "  ".bright_green(),
            "/model <N>".bright_green(),
            " to switch".bright_green(),
            "  ",
            config.ui.color_reset
        );
        print_panel_footer(config);
        println!();
    } else {
        // Switch model
        if let Ok(index) = args[0].parse::<usize>() {
            if index > 0 && index <= models.len() {
                let new_model = &models[index - 1];

                // Stop current server
                server.stop()?;

                // Start with new model
                server.ensure_running(config, Some(new_model)).await?;

                // Update current model name
                *current_model = new_model.clone();

                println!(
                    "{}✓ Switched to model: {}{}",
                    "  ".bright_green(),
                    new_model.bright_green(),
                    config.ui.color_reset
                );
                println!();
            } else {
                println!(
                    "{}✗ Invalid model index{}",
                    "  ".bright_red(),
                    config.ui.color_reset
                );
                println!();
            }
        } else {
            println!(
                "{}Usage: /model <number>{}",
                "  ".bright_red(),
                config.ui.color_reset
            );
            println!();
        }
    }

    Ok(())
}

fn get_available_models_list(config: &RuboxConfig) -> Vec<String> {
    let mut models = Vec::new();

    // Add from registry
    for name in config.models.registry.keys() {
        models.push(name.clone());
    }

    // Add from models directory
    if let Ok(entries) = fs::read_dir(Path::new("models")) {
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

// ============================================================================
// STYLING HELPERS
// ============================================================================

fn print_welcome_panel(_config: &RuboxConfig) {
    let orange = "\x1b[38;5;208m";
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    println!("{}{}{}", orange, "┌─────────────────────────────────────────────────────────┐", reset);
    println!("{}│{} Type / for commands                                   {}│{}",
        orange, emerald, orange, reset);
    println!("{}{}{}", orange, "└─────────────────────────────────────────────────────────┘", reset);
}

fn print_turn_header(model_name: &str, _config: &RuboxConfig) {
    let orange = "\x1b[38;5;208m";
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    let title = format!(" Turn – {} ", model_name);
    let padding = "─".repeat(60_usize.saturating_sub(title.len()));
    println!("{}{}{}{}{}", orange, emerald, title, reset, padding);
}

fn print_response_panel(response: &str, _config: &RuboxConfig) {
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    println!("{}┌─────────────────────────────────────────────────────────┐{}", emerald, reset);
    for line in response.lines() {
        println!("{}│ {:<57} │{}", emerald, line, reset);
    }
    println!("{}└─────────────────────────────────────────────────────────┘{}", emerald, reset);
}

fn print_stats_panel(tps: f32, elapsed_secs: f32, _config: &RuboxConfig) {
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    println!();
    println!("{}┌─ Performance Stats ──────────────────────────────────┐{}", emerald, reset);
    println!("{}│ Throughput: {:.1} tokens/sec                              │{}",
        emerald, tps, reset);
    println!("{}│ Response Time: {:.2}s                                    │{}",
        emerald, elapsed_secs, reset);
    println!("{}└──────────────────────────────────────────────────────┘{}", emerald, reset);
}

fn print_user_prompt(config: &RuboxConfig) -> anyhow::Result<()> {
    let orange = "\x1b[38;5;208m";
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    let user_label = format!("{} Input", config.user.name);
    let padding = "─".repeat(40_usize.saturating_sub(user_label.len()));
    println!("{}┌─ {}{} {}┐{}",
        orange, emerald, user_label, padding, reset);
    print!("{}", emerald);
    io::stdout().flush()?;
    Ok(())
}

fn print_panel_header(title: &str, _config: &RuboxConfig) {
    let orange = "\x1b[38;5;208m";
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    let padding = "─".repeat(57_usize.saturating_sub(title.len()));
    println!("{}┌─ {}{}{}{}{}", orange, emerald, title, orange, padding, reset);
}

fn print_panel_footer(_config: &RuboxConfig) {
    let orange = "\x1b[38;5;208m";
    let reset = "\x1b[0m";

    println!("{}{}{}", orange, "└─────────────────────────────────────────────────────────┘", reset);
}

// ============================================================================

fn save_chat(history: &[ChatMessage], model_name: &str, config: &RuboxConfig) -> anyhow::Result<()> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let chat_file = format!("{}/Chat_{}.md", config.directories.chat, timestamp);

    // Ensure directory exists
    fs::create_dir_all(&config.directories.chat)?;

    let mut markdown = String::new();
    let mut is_first = true;

    for msg in history {
        let speaker = if msg.role == "user" {
            config.user.name.clone()
        } else {
            model_name.to_string()
        };

        if !is_first {
            markdown.push_str("\n");
        }

        markdown.push_str(&format!("**{}**:\n{}\n", speaker, msg.content));
        is_first = false;
    }

    fs::write(&chat_file, markdown)?;
    println!();
    let orange = "\x1b[38;5;208m";
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";
    println!("{}{}{}", orange, "╔════════════════════════════════════════════════════╗", reset);
    println!("{}║ {}✓ Chat saved{}  {}║{}",
        orange, emerald, orange, " ".repeat(30), reset);
    println!("{}║ {}{}  {}║{}",
        orange, emerald, chat_file, " ".repeat(30_usize.saturating_sub(chat_file.len())), reset);
    println!("{}{}{}", orange, "╚════════════════════════════════════════════════════╝", reset);
    println!();

    Ok(())
}
