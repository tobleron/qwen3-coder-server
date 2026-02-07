use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Instant;

use crate::config::RuboxConfig;
use crate::llm_client::{LlmClient, ChatMessage as ApiChatMessage};
use crate::server_manager::ServerManager;
use crate::commands::{ChatState, CommandResult};
use crate::tui::{App, EventHandler, AppEvent, UIMode, ModalType};

pub async fn run_chat_mode(
    client: &LlmClient,
    model_name: &str,
    config: &RuboxConfig,
    _verbose: bool,
    server: &mut ServerManager,
) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(model_name.to_string(), config.temperature.default);

    // Create event handler
    let event_handler = EventHandler::new();
    let event_tx = event_handler.sender();

    // Create channel for LLM responses using tokio for async compatibility
    let (llm_tx, mut llm_rx) = tokio::sync::mpsc::unbounded_channel();

    // Main event loop
    while !app.should_exit {
        // Check for LLM response
        if let Ok(event) = llm_rx.try_recv() {
            match event {
                AppEvent::LlmResponse(text, usage) => {
                    app.add_assistant_message(text.clone(), usage.clone());
                    app.last_tps = if let Some(ref u) = usage {
                        u.completion_tokens as f32 / app.last_response_time
                    } else {
                        0.0
                    };

                    // Auto-save if enabled
                    if config.session.auto_save {
                        let _ = app.session.save(&config.directories.sessions);
                    }
                }
                AppEvent::LlmError(err) => {
                    app.set_error(err);
                }
                _ => {}
            }
        }

        // Render
        terminal.draw(|f| crate::tui::draw(f, &app))?;

        // Handle events
        if let Some(event) = event_handler.next() {
            match event {
                AppEvent::Key(key) => {
                    use crossterm::event::{KeyCode, KeyModifiers};

                    match key.code {
                        KeyCode::Char('/') if matches!(app.mode, UIMode::Chat) => {
                            // Only open command palette if typing / in chat mode
                            if app.input_buffer.is_empty() {
                                app.open_command_palette();
                            } else {
                                app.handle_input_char('/');
                            }
                        }
                        KeyCode::Esc => match app.mode {
                            UIMode::Chat => {
                                app.should_exit = true;
                            }
                            UIMode::CommandPalette => {
                                app.close_command_palette();
                            }
                            UIMode::Modal(_) => {
                                app.close_modal();
                            }
                        },
                        KeyCode::Enter => {
                            if let Some(input) = app.submit_input() {
                                if input.starts_with('/') {
                                    handle_command(
                                        input,
                                        &mut app,
                                        server,
                                        config,
                                        client,
                                        event_tx.clone(),
                                    )
                                    .await?;
                                } else if !input.is_empty() {
                                    app.session.add_message("user".to_string(), input, None);
                                    app.is_loading = true;

                                    // Spawn async LLM call
                                    let llm_tx = llm_tx.clone();
                                    let client = client.clone();
                                    let messages = app.session.messages.clone();
                                    let temp = app.temperature;

                                    tokio::spawn(async move {
                                        let api_messages: Vec<ApiChatMessage> = messages
                                            .iter()
                                            .map(|m| ApiChatMessage {
                                                role: m.role.clone(),
                                                content: m.content.clone(),
                                            })
                                            .collect();

                                        let start = Instant::now();
                                        match client.chat_completion_with_usage(api_messages, temp).await {
                                            Ok((response, usage)) => {
                                                let _elapsed = start.elapsed().as_secs_f32();
                                                let _ = llm_tx.send(AppEvent::LlmResponse(response, usage));
                                            }
                                            Err(e) => {
                                                let _ = llm_tx.send(AppEvent::LlmError(e.to_string()));
                                            }
                                        }
                                    });
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                match c {
                                    'c' => {
                                        app.should_exit = true;
                                    }
                                    _ => {}
                                }
                            } else {
                                app.handle_input_char(c);
                            }
                        }
                        KeyCode::Backspace => app.handle_backspace(),
                        KeyCode::Up => app.scroll_up(),
                        KeyCode::Down => app.scroll_down(),
                        _ => {}
                    }
                }
                AppEvent::Tick => app.tick(),
                AppEvent::Render => {
                    // Render happens in the main loop
                }
                _ => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

async fn handle_command(
    input: String,
    app: &mut App,
    server: &mut ServerManager,
    config: &RuboxConfig,
    client: &LlmClient,
    _event_tx: std::sync::mpsc::Sender<AppEvent>,
) -> anyhow::Result<()> {
    let mut state = ChatState {
        session: &mut app.session,
        current_model: &mut app.current_model,
        verbose: &mut false,
        temperature: &mut app.temperature,
        server,
        client,
        config,
    };

    match app.command_registry.handle(&input, &mut state)? {
        CommandResult::Exit => app.should_exit = true,
        CommandResult::SwitchModel(new_model) => {
            server.stop()?;
            server.ensure_running(config, Some(&new_model)).await?;
            app.current_model = new_model.clone();
        }
        CommandResult::Continue => {}
    }

    Ok(())
}
