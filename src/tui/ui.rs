use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::{App, UIMode, ModalType};

const ORANGE: Color = Color::Rgb(255, 135, 0);
const EMERALD: Color = Color::Rgb(0, 255, 135);

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Layout: [Chat History] [Input/Dropdown Area] [Status Bar]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Min(4), Constraint::Length(1)])
        .split(size);

    draw_chat_history(f, chunks[0], app);

    // Input area - can contain either input box, command palette, or modal
    match app.mode {
        UIMode::Chat => draw_input_box(f, chunks[1], app),
        UIMode::CommandPalette => draw_command_palette_dropdown(f, chunks[1], app),
        UIMode::Modal(_) => draw_modal_form(f, chunks[1], app),
    }

    draw_status_bar(f, chunks[2], app);
}

fn draw_chat_history(f: &mut Frame, area: Rect, app: &App) {
    let messages = app.get_visible_messages();
    let mut lines = Vec::new();

    for msg in messages {
        let role_style = if msg.role == "user" {
            Style::default().fg(ORANGE)
        } else {
            Style::default().fg(EMERALD)
        };

        let arrow = Span::styled("→ ", role_style);
        let role = Span::styled(&msg.role, role_style);
        lines.push(Line::from(vec![arrow, role]));

        // Simple text display
        for content_line in msg.content.lines() {
            lines.push(Line::from(content_line.to_string()));
        }

        lines.push(Line::from("")); // Empty line between messages
    }

    let block = Block::default()
        .title(format!(" Chat - {} ", app.current_model))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(EMERALD));

    let mut paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    if app.scroll_offset > 0 {
        paragraph = paragraph.scroll((app.scroll_offset as u16, 0));
    }

    f.render_widget(paragraph, area);
}

fn draw_input_box(f: &mut Frame, area: Rect, app: &App) {
    let input_text = if app.is_loading {
        format!("{}  {}", app.get_loading_spinner(), app.input_buffer)
    } else {
        app.input_buffer.clone()
    };

    let input_style = if app.is_loading {
        Style::default().fg(EMERALD)
    } else {
        Style::default().fg(Color::White)
    };

    let block = Block::default()
        .title(" Input (type / for commands) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(input_text)
        .block(block)
        .style(input_style)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

    // Set cursor position
    if !app.is_loading && area.height > 2 && area.width > 2 {
        let cursor_x = area.x + 1 + (app.input_buffer.len() as u16).min(area.width - 3);
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_command_palette_dropdown(f: &mut Frame, area: Rect, app: &App) {
    // Build the dropdown list
    let commands = app.get_filtered_commands();
    let mut lines = Vec::new();

    // Show the search input at top
    let search_input = format!("{}", app.command_search);
    let search_line = Line::from(vec![
        Span::raw("  "),
        Span::styled("/", Style::default().fg(ORANGE)),
        Span::styled(&search_input, Style::default().fg(EMERALD).add_modifier(Modifier::BOLD)),
        Span::raw("_"),
    ]);
    lines.push(search_line);
    lines.push(Line::from(""));

    // Add filtered commands
    if commands.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No commands found",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (idx, cmd) in commands.iter().enumerate() {
            let style = if idx == app.selected_command_idx {
                Style::default()
                    .bg(EMERALD)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(EMERALD)
            };

            let line_text = format!("  /{:<12} {}", cmd.name, cmd.help);
            lines.push(Line::from(Span::styled(line_text, style)));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

fn draw_modal_form(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = Vec::new();

    match app.mode {
        UIMode::Modal(ref modal_type) => {
            match modal_type {
                ModalType::ModelSelector => {
                    lines.push(Line::from(Span::styled(
                        "Select model: ",
                        Style::default().fg(ORANGE),
                    )));
                    lines.push(Line::from(""));
                    lines.push(Line::from(format!("  [1] {} ✓", app.current_model)));
                    lines.push(Line::from("  [2] Other model 1"));
                    lines.push(Line::from("  [3] Other model 2"));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "  Enter number or type to search",
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM),
                    )));
                }
                ModalType::SetTemperature => {
                    let current_temp = format!("Current: {:.1}", app.temperature);
                    lines.push(Line::from(vec![
                        Span::styled("Set temperature (0.0-2.0): ", Style::default().fg(ORANGE)),
                        Span::styled(
                            current_temp,
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                    lines.push(Line::from(""));

                    let input_str = format!("  {}_", app.modal_input);
                    lines.push(Line::from(Span::styled(
                        input_str,
                        Style::default().fg(EMERALD),
                    )));
                }
                ModalType::DeleteMessage => {
                    lines.push(Line::from(Span::styled(
                        "Delete message (enter ID or 'all'): ",
                        Style::default().fg(ORANGE),
                    )));
                    lines.push(Line::from(""));

                    let input_str = format!("  {}_", app.modal_input);
                    lines.push(Line::from(Span::styled(
                        input_str,
                        Style::default().fg(EMERALD),
                    )));
                }
                ModalType::SaveResponse => {
                    lines.push(Line::from(Span::styled(
                        "Save response (enter message ID): ",
                        Style::default().fg(ORANGE),
                    )));
                    lines.push(Line::from(""));

                    let input_str = format!("  {}_", app.modal_input);
                    lines.push(Line::from(Span::styled(
                        input_str,
                        Style::default().fg(EMERALD),
                    )));
                }
                ModalType::RenameSession => {
                    lines.push(Line::from(Span::styled(
                        "Rename session (enter new label): ",
                        Style::default().fg(ORANGE),
                    )));
                    lines.push(Line::from(""));

                    let input_str = format!("  {}_", app.modal_input);
                    lines.push(Line::from(Span::styled(
                        input_str,
                        Style::default().fg(EMERALD),
                    )));
                }
                ModalType::LoadPrompt => {
                    lines.push(Line::from(Span::styled(
                        "Load prompt (enter ID or 'list'): ",
                        Style::default().fg(ORANGE),
                    )));
                    lines.push(Line::from(""));

                    let input_str = format!("  {}_", app.modal_input);
                    lines.push(Line::from(Span::styled(
                        input_str,
                        Style::default().fg(EMERALD),
                    )));
                }
            }
        }
        _ => {}
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

    // Set cursor position in modal
    if area.height > 2 && area.width > 2 {
        let cursor_x = area.x + 3 + (app.modal_input.len() as u16).min(area.width - 5);
        let cursor_y = area.y + 3; // Approximate position
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let status_text = if let Some(error) = &app.error_message {
        format!("✗ Error: {}", error)
    } else if app.is_loading {
        format!("⚡ Loading... ({})", app.get_loading_spinner())
    } else {
        format!(
            "⚡ {:.1} tps | {:.2}s | Temp: {:.1}",
            app.last_tps, app.last_response_time, app.temperature
        )
    };

    let status_style = if app.error_message.is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(EMERALD)
    };

    let paragraph = Paragraph::new(status_text)
        .style(status_style)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}
