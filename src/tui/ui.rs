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
    match app.mode {
        UIMode::Chat => draw_chat_view(f, app),
        UIMode::CommandPalette => draw_command_palette(f, app),
        UIMode::Modal(ref modal_type) => draw_modal(f, app, modal_type),
    }
}

fn draw_chat_view(f: &mut Frame, app: &App) {
    let size = f.area();

    // Layout: [Chat History] [Input Box] [Status Bar]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(4), Constraint::Length(1)])
        .split(size);

    draw_chat_history(f, chunks[0], app);
    draw_input_box(f, chunks[1], app);
    draw_status_bar(f, chunks[2], app);
}

fn draw_command_palette(f: &mut Frame, app: &App) {
    let size = f.area();

    // Center the command palette
    let width = 60;
    let height = 16;
    let x = (size.width.saturating_sub(width)) / 2;
    let y = (size.height.saturating_sub(height)) / 2;
    let modal_area = Rect {
        x,
        y,
        width,
        height,
    };

    // Draw semi-transparent background
    let background = Block::default()
        .borders(Borders::NONE)
        .style(Style::default());
    f.render_widget(background, size);

    // Build command list
    let commands = app.get_filtered_commands();
    let mut items: Vec<Line> = Vec::new();

    // Add search box at top
    items.push(Line::from(vec![
        Span::raw("Search: "),
        Span::styled(&app.command_search, Style::default().fg(EMERALD)),
        Span::raw("_"),
    ]));
    items.push(Line::from(""));

    // Add commands
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
        items.push(Line::from(Span::styled(line_text, style)));
    }

    items.push(Line::from(""));
    items.push(Line::from(Span::styled(
        "↑↓ Navigate  Enter Select  Esc Cancel",
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM),
    )));

    let block = Block::default()
        .title(" Commands ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(items)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, modal_area);
}

fn draw_modal(f: &mut Frame, app: &App, modal_type: &ModalType) {
    let size = f.area();

    // Center the modal
    let width = 50;
    let height = 12;
    let x = (size.width.saturating_sub(width)) / 2;
    let y = (size.height.saturating_sub(height)) / 2;
    let modal_area = Rect {
        x,
        y,
        width,
        height,
    };

    match modal_type {
        ModalType::ModelSelector => draw_modal_model_selector(f, app, modal_area),
        ModalType::SetTemperature => draw_modal_temperature(f, app, modal_area),
        ModalType::DeleteMessage => draw_modal_delete_message(f, app, modal_area),
        ModalType::SaveResponse => draw_modal_save_response(f, app, modal_area),
        ModalType::RenameSession => draw_modal_rename_session(f, app, modal_area),
        ModalType::LoadPrompt => draw_modal_load_prompt(f, app, modal_area),
    }
}

fn draw_modal_model_selector(f: &mut Frame, app: &App, area: Rect) {
    let mut items = Vec::new();
    items.push(Line::from(Span::styled(
        "Select Model:",
        Style::default().fg(ORANGE),
    )));
    items.push(Line::from(""));

    // Display available models (placeholder - would need model list from config)
    items.push(Line::from(format!("  [1] {} ✓", app.current_model)));
    items.push(Line::from("  [2] Other models..."));

    items.push(Line::from(""));
    items.push(Line::from(Span::styled(
        "Enter model number",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .title(" Model ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}

fn draw_modal_temperature(f: &mut Frame, app: &App, area: Rect) {
    let mut items = Vec::new();
    items.push(Line::from(Span::styled(
        "Set Temperature:",
        Style::default().fg(ORANGE),
    )));
    items.push(Line::from(""));
    items.push(Line::from(format!(
        "Current: {:.1}  (Range: 0.0-2.0)",
        app.temperature
    )));
    items.push(Line::from(""));

    let input_str = format!("Input: {}_", app.modal_input);
    items.push(Line::from(Span::styled(
        input_str,
        Style::default().fg(EMERALD),
    )));
    items.push(Line::from(""));
    items.push(Line::from(Span::styled(
        "Enter new value",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .title(" Temperature ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}

fn draw_modal_delete_message(f: &mut Frame, app: &App, area: Rect) {
    let mut items = Vec::new();
    items.push(Line::from(Span::styled(
        "Delete Message:",
        Style::default().fg(ORANGE),
    )));
    items.push(Line::from(""));
    items.push(Line::from("Enter message ID or 'all'"));
    items.push(Line::from(""));

    let input_str = format!("Input: {}_", app.modal_input);
    items.push(Line::from(Span::styled(
        input_str,
        Style::default().fg(EMERALD),
    )));

    let block = Block::default()
        .title(" Delete ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}

fn draw_modal_save_response(f: &mut Frame, app: &App, area: Rect) {
    let mut items = Vec::new();
    items.push(Line::from(Span::styled(
        "Save Response:",
        Style::default().fg(ORANGE),
    )));
    items.push(Line::from(""));
    items.push(Line::from("Enter message ID to export"));
    items.push(Line::from(""));

    let input_str = format!("Input: {}_", app.modal_input);
    items.push(Line::from(Span::styled(
        input_str,
        Style::default().fg(EMERALD),
    )));

    let block = Block::default()
        .title(" Save ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}

fn draw_modal_rename_session(f: &mut Frame, app: &App, area: Rect) {
    let mut items = Vec::new();
    items.push(Line::from(Span::styled(
        "Rename Session:",
        Style::default().fg(ORANGE),
    )));
    items.push(Line::from(""));
    items.push(Line::from("Enter new label for this session"));
    items.push(Line::from(""));

    let input_str = format!("Input: {}_", app.modal_input);
    items.push(Line::from(Span::styled(
        input_str,
        Style::default().fg(EMERALD),
    )));

    let block = Block::default()
        .title(" Rename ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}

fn draw_modal_load_prompt(f: &mut Frame, app: &App, area: Rect) {
    let mut items = Vec::new();
    items.push(Line::from(Span::styled(
        "Load Prompt:",
        Style::default().fg(ORANGE),
    )));
    items.push(Line::from(""));
    items.push(Line::from("Enter prompt ID or 'list'"));
    items.push(Line::from(""));

    let input_str = format!("Input: {}_", app.modal_input);
    items.push(Line::from(Span::styled(
        input_str,
        Style::default().fg(EMERALD),
    )));

    let block = Block::default()
        .title(" Prompt ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
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

        // Simple text display (no markdown rendering complexity)
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
        .title(" Input (/ for commands) ")
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
