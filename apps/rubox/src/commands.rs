use anyhow::Result;
use colored::*;
use crate::config::RuboxConfig;
use crate::session::Session;
use crate::server_manager::ServerManager;
use crate::llm_client::LlmClient;
use crate::prompts::PromptManager;
use std::fs;

pub enum CommandResult {
    Continue,
    Exit,
    SwitchModel(String),  // Signal to switch to a new model
}

pub struct ChatState<'a> {
    pub session: &'a mut Session,
    pub current_model: &'a mut String,
    pub verbose: &'a mut bool,
    pub temperature: &'a mut f32,
    #[allow(dead_code)]
    pub server: &'a mut ServerManager,
    #[allow(dead_code)]
    pub client: &'a LlmClient,
    pub config: &'a RuboxConfig,
}

pub type CommandHandler = fn(&mut ChatState, &[&str]) -> Result<CommandResult>;

pub struct Command {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub handler: CommandHandler,
    pub help: &'static str,
}

pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        CommandRegistry {
            commands: vec![
                Command {
                    name: "help",
                    aliases: &["h", "?"],
                    handler: cmd_help,
                    help: "Show this help message",
                },
                Command {
                    name: "exit",
                    aliases: &["quit", "q"],
                    handler: cmd_exit,
                    help: "Save and exit chat",
                },
                Command {
                    name: "verbose",
                    aliases: &["v"],
                    handler: cmd_verbose,
                    help: "Toggle verbose mode (show stats)",
                },
                Command {
                    name: "model",
                    aliases: &["m"],
                    handler: cmd_model,
                    help: "List or switch models",
                },
                Command {
                    name: "history",
                    aliases: &["hist"],
                    handler: cmd_history,
                    help: "Show conversation history",
                },
                Command {
                    name: "delete",
                    aliases: &["del", "rm"],
                    handler: cmd_delete,
                    help: "Delete message(s): /delete <id|all>",
                },
                Command {
                    name: "save",
                    aliases: &["export"],
                    handler: cmd_save,
                    help: "Save response: /save <id>",
                },
                Command {
                    name: "set",
                    aliases: &[],
                    handler: cmd_set,
                    help: "Set parameter: /set temp <value>",
                },
                Command {
                    name: "temp",
                    aliases: &["temperature"],
                    handler: cmd_temp,
                    help: "Show current temperature",
                },
                Command {
                    name: "sessions",
                    aliases: &["sess"],
                    handler: cmd_sessions,
                    help: "List all sessions",
                },
                Command {
                    name: "load",
                    aliases: &[],
                    handler: cmd_load,
                    help: "Load session: /load <id>",
                },
                Command {
                    name: "rename",
                    aliases: &[],
                    handler: cmd_rename,
                    help: "Rename session: /rename <label>",
                },
                Command {
                    name: "prompt",
                    aliases: &["p"],
                    handler: cmd_prompt,
                    help: "Load static prompt: /prompt <id|list>",
                },
            ],
        }
    }

    pub fn handle(&self, input: &str, state: &mut ChatState) -> Result<CommandResult> {
        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        let command = parts.first().unwrap_or(&"");

        if command.is_empty() {
            return cmd_help(state, &[]);
        }

        for cmd in &self.commands {
            if cmd.name == *command || cmd.aliases.contains(command) {
                return (cmd.handler)(state, &parts[1..]);
            }
        }

        println!("  {}✗ Unknown command: /{}. Type / for help.",
            "".bright_red(), command);
        Ok(CommandResult::Continue)
    }

    #[allow(dead_code)]
    pub fn get_help_text(&self) -> String {
        let mut help = String::from("Available Commands:\n");
        for cmd in &self.commands {
            help.push_str(&format!("  /{:<15} - {}\n", cmd.name, cmd.help));
        }
        help
    }

    pub fn get_all_commands(&self) -> &[Command] {
        &self.commands
    }
}

// ============================================================================
// COMMAND IMPLEMENTATIONS
// ============================================================================

fn cmd_help(_state: &mut ChatState, _args: &[&str]) -> Result<CommandResult> {
    println!();
    let orange = "\x1b[38;5;208m";
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    let padding = "─".repeat(57);
    println!("{}┌─ {}Available Commands{}{}{}", orange, emerald, orange, padding, reset);

    for cmd in &CommandRegistry::new().commands {
        println!("{}  {:<15} {}─ {}{}",
            "  ".bright_green(),
            format!("/{}", cmd.name),
            "─".bright_green(),
            cmd.help,
            reset);
    }

    println!("{}{}{}", orange, "└─────────────────────────────────────────────────────────┘", reset);
    println!();
    Ok(CommandResult::Continue)
}

fn cmd_exit(state: &mut ChatState, _args: &[&str]) -> Result<CommandResult> {
    state.session.save(&state.config.directories.sessions)?;
    println!();
    let orange = "\x1b[38;5;208m";
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";
    println!("{}{}{}", orange, "╔════════════════════════════════════════════════════╗", reset);
    println!("{}║ {}✓ Chat saved and session persisted{}  {}║{}",
        orange, emerald, orange, " ".repeat(20), reset);
    println!("{}{}{}", orange, "╚════════════════════════════════════════════════════╝", reset);
    println!();
    Ok(CommandResult::Exit)
}

fn cmd_verbose(state: &mut ChatState, _args: &[&str]) -> Result<CommandResult> {
    *state.verbose = !*state.verbose;
    let status = if *state.verbose {
        "✓ Verbose mode enabled".bright_green()
    } else {
        "✗ Verbose mode disabled".bright_red()
    };
    println!("  {}", status);
    println!();
    Ok(CommandResult::Continue)
}

fn cmd_model(state: &mut ChatState, args: &[&str]) -> Result<CommandResult> {
    // Get registry keys (canonical model names)
    let mut models: Vec<String> = state.config.models.registry.keys().cloned().collect();
    models.sort();

    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    if args.is_empty() {
        // List models
        println!();
        println!("{}Available Models:{}", emerald, reset);
        for (i, model) in models.iter().enumerate() {
            let current_marker = if model == state.current_model {
                " ✓".to_string()
            } else {
                String::new()
            };
            println!("  [{}] {}{}", i + 1, model, current_marker);
        }
        println!();
        println!("{}→ /model <N> to switch{}", emerald, reset);
        println!();
    } else {
        // Switch model
        if let Ok(index) = args[0].parse::<usize>() {
            if index > 0 && index <= models.len() {
                let new_model = &models[index - 1];
                println!("  {}⏳ Switching to {}...{}", emerald, new_model, reset);

                // Return signal to switch model - chat loop will handle async restart
                return Ok(CommandResult::SwitchModel(new_model.clone()));
            } else {
                println!(
                    "  {}✗ Invalid model index{}",
                    "".bright_red(),
                    state.config.ui.color_reset
                );
                println!();
            }
        } else {
            println!(
                "  {}Usage: /model <number>{}",
                "".bright_red(),
                state.config.ui.color_reset
            );
            println!();
        }
    }

    Ok(CommandResult::Continue)
}

fn cmd_history(state: &mut ChatState, _args: &[&str]) -> Result<CommandResult> {
    println!();
    let orange = "\x1b[38;5;208m";
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    let padding = "─".repeat(57_usize.saturating_sub("Conversation History".len()));
    println!("{}┌─ {}Conversation History{}{}{}", orange, emerald, orange, padding, reset);

    for msg in &state.session.messages {
        let speaker = if msg.role == "user" {
            &state.config.user.name
        } else {
            state.current_model.as_str()
        };

        let preview = msg.content.chars().take(50).collect::<String>();
        println!("{}  [{}] {}: {}...",
            "  ".bright_green(),
            msg.id,
            speaker.bright_green(),
            preview);
    }

    println!("{}{}{}", orange, "└─────────────────────────────────────────────────────────┘", reset);
    println!();
    Ok(CommandResult::Continue)
}

fn cmd_delete(state: &mut ChatState, args: &[&str]) -> Result<CommandResult> {
    if args.is_empty() {
        println!("  {}Usage: /delete <id|all>", "".bright_red());
        return Ok(CommandResult::Continue);
    }

    if args[0] == "all" {
        state.session.clear_all();
        println!("  {}✓ All messages deleted", "".bright_green());
    } else {
        let id: usize = args[0].parse()?;
        state.session.delete_message(id)?;
        println!("  {}✓ Message {} deleted", "".bright_green(), id);
    }

    state.session.save(&state.config.directories.sessions)?;
    Ok(CommandResult::Continue)
}

fn cmd_save(state: &mut ChatState, args: &[&str]) -> Result<CommandResult> {
    if args.is_empty() {
        println!("  {}Usage: /save <id>", "".bright_red());
        return Ok(CommandResult::Continue);
    }

    let id: usize = args[0].parse()?;

    if let Some(msg) = state.session.get_message(id) {
        let save_dir = &state.config.directories.saved_responses;
        fs::create_dir_all(save_dir)?;

        let filename = format!("{}/{}_{}.txt",
            save_dir,
            state.session.metadata.id,
            id);

        fs::write(&filename, &msg.content)?;
        println!("  {}✓ Saved to: {}", "".bright_green(), filename);
    } else {
        println!("  {}✗ Message {} not found", "".bright_red(), id);
    }

    Ok(CommandResult::Continue)
}

fn cmd_set(state: &mut ChatState, args: &[&str]) -> Result<CommandResult> {
    if args.len() < 2 {
        println!("  {}Usage: /set temp <value>", "".bright_red());
        return Ok(CommandResult::Continue);
    }

    match args[0] {
        "temp" | "temperature" => {
            let value: f32 = args[1].parse()?;

            if value < state.config.temperature.min || value > state.config.temperature.max {
                println!("  {}✗ Temperature must be between {} and {}",
                    "".bright_red(),
                    state.config.temperature.min,
                    state.config.temperature.max);
                return Ok(CommandResult::Continue);
            }

            *state.temperature = value;
            state.session.metadata.temperature = value;
            println!("  {}✓ Temperature set to {}", "".bright_green(), value);
        }
        _ => {
            println!("  {}✗ Unknown parameter: {}", "".bright_red(), args[0]);
        }
    }

    Ok(CommandResult::Continue)
}

fn cmd_temp(state: &mut ChatState, _args: &[&str]) -> Result<CommandResult> {
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";
    println!("  {}Current temperature: {:.1}{}", emerald, state.temperature, reset);
    println!();
    Ok(CommandResult::Continue)
}

fn cmd_sessions(state: &mut ChatState, _args: &[&str]) -> Result<CommandResult> {
    let sessions = Session::list_sessions(&state.config.directories.sessions)?;

    println!();
    let orange = "\x1b[38;5;208m";
    let emerald = "\x1b[38;5;48m";
    let reset = "\x1b[0m";

    let padding = "─".repeat(57_usize.saturating_sub("Available Sessions".len()));
    println!("{}┌─ {}Available Sessions{}{}{}", orange, emerald, orange, padding, reset);

    for (i, session) in sessions.iter().enumerate() {
        let label = session.label.as_ref()
            .map(|l| format!(" ({})", l))
            .unwrap_or_default();

        println!("{}  [{}] {}{}  {} messages, {} tokens",
            "  ".bright_green(),
            i + 1,
            session.id.bright_green(),
            label,
            session.message_count,
            session.total_tokens);
    }

    println!("{}{}{}", orange, "└─────────────────────────────────────────────────────────┘", reset);
    println!();
    Ok(CommandResult::Continue)
}

fn cmd_load(_state: &mut ChatState, _args: &[&str]) -> Result<CommandResult> {
    println!("  {}Note: To load a session, restart the application with that session in the Chat/sessions directory", "".bright_green());
    Ok(CommandResult::Continue)
}

fn cmd_rename(state: &mut ChatState, args: &[&str]) -> Result<CommandResult> {
    if args.is_empty() {
        println!("  {}Usage: /rename <label>", "".bright_red());
        return Ok(CommandResult::Continue);
    }

    let label = args.join("_");
    state.session.rename(label.clone());
    state.session.save(&state.config.directories.sessions)?;

    println!("  {}✓ Session renamed to: {}",
        "".bright_green(),
        state.session.metadata.id);

    Ok(CommandResult::Continue)
}

fn cmd_prompt(state: &mut ChatState, args: &[&str]) -> Result<CommandResult> {
    let pm = PromptManager::new(state.config.directories.static_prompts.clone());

    if args.is_empty() || args[0] == "list" {
        let prompts = pm.list_prompts()?;

        println!();
        let orange = "\x1b[38;5;208m";
        let emerald = "\x1b[38;5;48m";
        let reset = "\x1b[0m";

        let padding = "─".repeat(57_usize.saturating_sub("Static Prompts".len()));
        println!("{}┌─ {}Static Prompts{}{}{}", orange, emerald, orange, padding, reset);

        if prompts.is_empty() {
            println!("{}  No prompts available", "  ".bright_green());
        } else {
            for (i, prompt) in prompts.iter().enumerate() {
                println!("{}  [{}] {}",
                    "  ".bright_green(),
                    i + 1,
                    prompt.bright_green());
            }
        }

        println!("{}{}{}", orange, "└─────────────────────────────────────────────────────────┘", reset);
        println!();
    } else {
        let prompts = pm.list_prompts()?;
        let idx: usize = args[0].parse::<usize>()? - 1;

        if idx >= prompts.len() {
            println!("  {}✗ Invalid prompt index", "".bright_red());
            return Ok(CommandResult::Continue);
        }

        let content = pm.load_prompt(&prompts[idx])?;

        println!();
        println!("{}Loaded prompt: {}", "  ".bright_green(), prompts[idx]);
        println!("{}{}", "  ".bright_green(), content);
        println!();

        // Add to session as user message
        state.session.add_message("user".to_string(), content, None);
    }

    Ok(CommandResult::Continue)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

