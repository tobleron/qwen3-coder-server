use std::io::{self, Write};
use crate::config::RuboxConfig;

pub fn display_colored(message: &str, color: &str, reset: &str) {
    println!("{}{}{}", color, message, reset);
}

pub fn display_model_list(models: &[String], config: &RuboxConfig) {
    println!();
    println!(
        "{}═══════════════════════════════════════{}",
        config.ui.color_orange, config.ui.color_reset
    );
    println!(
        "{}Available models:{}",
        config.ui.color_orange, config.ui.color_reset
    );
    println!(
        "{}═══════════════════════════════════════{}",
        config.ui.color_orange, config.ui.color_reset
    );
    for (i, model) in models.iter().enumerate() {
        println!("  {}[{}] {}{}",
            config.ui.color_orange, i + 1, model, config.ui.color_reset);
    }
    println!();
}

pub fn read_model_selection(models: &[String]) -> anyhow::Result<Vec<usize>> {
    let orange = "\x1b[38;5;208m";
    let reset = "\x1b[0m";
    println!("{}Choose Model (Example: 1 for chat-mode and 1,2 for multiple responses):{}", orange, reset);
    print!("{}", orange);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    println!("{}", reset);
    println!();

    let mut selected = Vec::new();
    for index_str in input.trim().split(',') {
        if let Ok(index) = index_str.trim().parse::<usize>() {
            if index > 0 && index <= models.len() {
                selected.push(index - 1);
            }
        }
    }

    if selected.is_empty() {
        anyhow::bail!("Invalid model selection");
    }

    Ok(selected)
}

pub fn get_user_input(prompt: &str) -> anyhow::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
