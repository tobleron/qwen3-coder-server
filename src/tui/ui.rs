use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::{App, UIMode, ModalType};

const ORANGE: Color = Color::Rgb(255, 135, 0);
const EMERALD: Color = Color::Rgb(0, 255, 135);
const GRAY: Color = Color::Rgb(128, 128, 128);
const DARK_GRAY: Color = Color::Rgb(60, 60, 60);

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Main layout with clean proportions
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Chat area
            Constraint::Min(10),         // Input/command area
            Constraint::Length(1),       // Status bar
        ])
        .split(size);

    // Always draw chat history
    draw_chat_history(f, chunks[0], app);

    // Draw appropriate mode
    match app.mode {
        UIMode::Chat => draw_input_normal(f, chunks[1], app),
        UIMode::CommandPalette => draw_command_list(f, chunks[1], app),
        UIMode::Modal(ref modal_type) => draw_modal_form(f, chunks[1], app, modal_type),
    }

    // Status bar
    draw_status_bar(f, chunks[2], app);
}

fn draw_chat_history(f: &mut Frame, area: Rect, app: &App) {
    let messages = app.get_visible_messages();
    let mut lines = Vec::new();

    if messages.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Start chatting by typing a message below",
            Style::default().fg(GRAY),
        )));
        lines.push(Line::from(Span::styled(
            "  Press / to see available commands",
            Style::default().fg(GRAY),
        )));
    } else {
        for msg in messages {
            lines.push(Line::from(""));

            let role_text = if msg.role == "user" { "You" } else { &app.current_model };
            lines.push(Line::from(Span::styled(
                role_text,
                Style::default()
                    .fg(if msg.role == "user" { ORANGE } else { EMERALD })
                    .add_modifier(Modifier::BOLD),
            )));

            // Content with proper wrapping
            for content_line in msg.content.lines() {
                lines.push(Line::from(format!("  {}", content_line)));
            }
        }
    }

    let block = Block::default()
        .title(format!(" {} ", app.current_model))
        .title_style(Style::default().fg(EMERALD).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GRAY));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset as u16, 0));

    f.render_widget(paragraph, area);
}

fn draw_input_normal(f: &mut Frame, area: Rect, app: &App) {
    let input_text = if app.is_loading {
        format!("  {}  Generating response...", app.get_loading_spinner())
    } else {
        format!("  {}", app.input_buffer)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GRAY));

    let style = if app.is_loading {
        Style::default().fg(GRAY)
    } else {
        Style::default().fg(Color::White)
    };

    let paragraph = Paragraph::new(input_text)
        .block(block)
        .style(style)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);

    // Cursor position
    if !app.is_loading && area.height > 2 && area.width > 4 {
        let cursor_x = area.x + 3 + (app.input_buffer.len() as u16).min(area.width - 5);
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_command_list(f: &mut Frame, area: Rect, app: &App) {
    let commands = app.get_filtered_commands();
    let mut items = Vec::new();

    // Search header
    let search_text = format!("/{}", app.command_search);
    items.push(ListItem::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(search_text, Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
    ])));

    items.push(ListItem::new(Line::from("")));

    // Command list
    if commands.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  No matching commands",
            Style::default().fg(GRAY),
        ))));
    } else {
        for (idx, cmd) in commands.iter().enumerate() {
            let is_selected = idx == app.selected_command_idx;

            let line = if is_selected {
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("▶ ", Style::default().fg(ORANGE)),
                    Span::styled(
                        format!("{:<12}", cmd.name),
                        Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(cmd.help, Style::default().fg(EMERALD)),
                ])
            } else {
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(format!("{:<12}", cmd.name), Style::default().fg(EMERALD)),
                    Span::raw(" "),
                    Span::styled(cmd.help, Style::default().fg(GRAY)),
                ])
            };

            items.push(ListItem::new(line));
        }
    }

    // Footer
    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "  ↑↓ navigate  •  enter select  •  esc cancel",
        Style::default().fg(DARK_GRAY),
    ))));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_modal_form(f: &mut Frame, area: Rect, app: &App, modal_type: &ModalType) {
    let mut items = Vec::new();

    match modal_type {
        ModalType::ModelSelector => {
            items.push(ListItem::new(Line::from(Span::styled(
                "  Select Model",
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
            ))));
            items.push(ListItem::new(Line::from("")));

            let mut models: Vec<_> = app.model_registry.iter().collect();
            models.sort_by(|a, b| a.0.cmp(b.0));

            if models.is_empty() {
                items.push(ListItem::new(Line::from(Span::styled(
                    "  No models available",
                    Style::default().fg(GRAY),
                ))));
            } else {
                for (idx, (name, path)) in models.iter().enumerate() {
                    let is_selected = idx == app.selected_model_idx;
                    let is_current = name.as_str() == app.current_model.as_str();

                    // Model name with selection indicator
                    let name_line = if is_selected {
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled("▶ ", Style::default().fg(ORANGE)),
                            Span::styled(
                                name.as_str(),
                                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                if is_current { "  ✓" } else { "" },
                                Style::default().fg(ORANGE),
                            ),
                        ])
                    } else {
                        Line::from(vec![
                            Span::raw("    "),
                            Span::styled(
                                name.as_str(),
                                Style::default().fg(if is_current { ORANGE } else { EMERALD }),
                            ),
                            Span::styled(
                                if is_current { "  ✓" } else { "" },
                                Style::default().fg(ORANGE),
                            ),
                        ])
                    };
                    items.push(ListItem::new(name_line));

                    // Full path in gray
                    let path_line = Line::from(vec![
                        Span::raw("     "),
                        Span::styled(path.as_str(), Style::default().fg(GRAY)),
                    ]);
                    items.push(ListItem::new(path_line));

                    items.push(ListItem::new(Line::from("")));
                }
            }

            items.push(ListItem::new(Line::from(Span::styled(
                "  ↑↓ navigate  •  enter select  •  esc cancel",
                Style::default().fg(DARK_GRAY),
            ))));
        }
        ModalType::SetTemperature => {
            items.push(ListItem::new(Line::from(Span::styled(
                "  Set Temperature",
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
            ))));
            items.push(ListItem::new(Line::from("")));

            let current_text = format!("Current: {:.1}  (Range: 0.0 - 2.0)", app.temperature);
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(current_text, Style::default().fg(GRAY)),
            ])));
            items.push(ListItem::new(Line::from("")));

            let input_line = Line::from(vec![
                Span::raw("  > "),
                Span::styled(&app.modal_input, Style::default().fg(EMERALD)),
                Span::raw("_"),
            ]);
            items.push(ListItem::new(input_line));
        }
        ModalType::DeleteMessage => {
            items.push(ListItem::new(Line::from(Span::styled(
                "  Delete Message",
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
            ))));
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from("  Enter message ID or 'all'")));
            items.push(ListItem::new(Line::from("")));

            let input_line = Line::from(vec![
                Span::raw("  > "),
                Span::styled(&app.modal_input, Style::default().fg(EMERALD)),
                Span::raw("_"),
            ]);
            items.push(ListItem::new(input_line));
        }
        ModalType::SaveResponse => {
            items.push(ListItem::new(Line::from(Span::styled(
                "  Save Response",
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
            ))));
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from("  Enter message ID to export")));
            items.push(ListItem::new(Line::from("")));

            let input_line = Line::from(vec![
                Span::raw("  > "),
                Span::styled(&app.modal_input, Style::default().fg(EMERALD)),
                Span::raw("_"),
            ]);
            items.push(ListItem::new(input_line));
        }
        ModalType::RenameSession => {
            items.push(ListItem::new(Line::from(Span::styled(
                "  Rename Session",
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
            ))));
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from("  Enter new label for this session")));
            items.push(ListItem::new(Line::from("")));

            let input_line = Line::from(vec![
                Span::raw("  > "),
                Span::styled(&app.modal_input, Style::default().fg(EMERALD)),
                Span::raw("_"),
            ]);
            items.push(ListItem::new(input_line));
        }
        ModalType::LoadPrompt => {
            items.push(ListItem::new(Line::from(Span::styled(
                "  Load Prompt",
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
            ))));
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from("  Enter prompt ID or 'list'")));
            items.push(ListItem::new(Line::from("")));

            let input_line = Line::from(vec![
                Span::raw("  > "),
                Span::styled(&app.modal_input, Style::default().fg(EMERALD)),
                Span::raw("_"),
            ]);
            items.push(ListItem::new(input_line));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let status_text = if let Some(error) = &app.error_message {
        format!(" ✗ Error: {}", error)
    } else if app.is_loading {
        format!(
            " {} Generating  │  Temp: {:.1}",
            app.get_loading_spinner(),
            app.temperature
        )
    } else {
        format!(
            " ⚡ {:.1} tok/s  │  {:.2}s  │  🌡️  {:.1}  │  {} messages",
            app.last_tps,
            app.last_response_time,
            app.temperature,
            app.session.messages.len()
        )
    };

    let style = if app.error_message.is_some() {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if app.is_loading {
        Style::default().fg(EMERALD)
    } else {
        Style::default().fg(GRAY)
    };

    let paragraph = Paragraph::new(status_text)
        .style(style)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}
