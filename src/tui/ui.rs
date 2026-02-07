use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::App;

const ORANGE: Color = Color::Rgb(255, 135, 0);
const EMERALD: Color = Color::Rgb(0, 255, 135);

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    if app.drawer_open {
        // Split layout: [Drawer 30%] [Chat 70%]
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(size);

        draw_command_drawer(f, chunks[0], app);
        draw_chat_area(f, chunks[1], app);
    } else {
        // Full width chat
        draw_chat_area(f, size, app);
    }
}

fn draw_command_drawer(f: &mut Frame, area: Rect, app: &App) {
    let commands = app.command_registry.get_all_commands();

    let mut items = Vec::new();
    for (idx, cmd) in commands.iter().enumerate() {
        let style = if idx == app.selected_command {
            Style::default()
                .fg(ORANGE)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(EMERALD)
        };

        let cmd_line = format!("  /{:<12} - {}", cmd.name, cmd.help);
        items.push(Line::from(Span::styled(cmd_line, style)));
    }

    let drawer_block = Block::default()
        .title(" Commands (Tab to close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ORANGE));

    let paragraph = Paragraph::new(items)
        .block(drawer_block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

fn draw_chat_area(f: &mut Frame, area: Rect, app: &App) {
    // Vertical layout: [History] [Input 4 lines] [Status 1 line]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(area);

    draw_chat_history(f, chunks[0], app);
    draw_input_box(f, chunks[1], app);
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

        // Format message content (markdown support)
        let formatted_content = format_markdown_for_ratatui(&msg.content);
        lines.extend(formatted_content);

        lines.push(Line::from("")); // Empty line between messages
    }

    let block = Block::default()
        .title(format!(
            " Chat - {} (Tab for commands) ",
            app.current_model
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(EMERALD));

    let mut paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    // Apply scroll offset
    if app.scroll_offset > 0 {
        paragraph = paragraph.scroll((app.scroll_offset as u16, 0));
    }

    f.render_widget(paragraph, area);
}

fn draw_input_box(f: &mut Frame, area: Rect, app: &App) {
    let input_text = if app.is_loading {
        format!(
            "{}  {}",
            app.get_loading_spinner(),
            app.input_buffer
        )
    } else {
        app.input_buffer.clone()
    };

    let input_style = if app.is_loading {
        Style::default().fg(EMERALD)
    } else {
        Style::default().fg(Color::White)
    };

    let block = Block::default()
        .title(" Input ")
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
        format!(
            "⚡ Loading... ({})",
            app.get_loading_spinner()
        )
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

fn format_markdown_for_ratatui(text: &str) -> Vec<Line<'static>> {
    // For simplicity and to avoid lifetime issues, just return basic lines
    // Actual markdown rendering would be more complex with owned data
    let lines: Vec<Line<'static>> = text
        .lines()
        .map(|line| Line::from(line.to_string()))
        .collect();

    if lines.is_empty() {
        vec![Line::from("")]
    } else {
        lines
    }
}
