use crate::session::Session;
use crate::commands::CommandRegistry;

pub struct App {
    // Session data
    pub session: Session,
    pub current_model: String,
    pub temperature: f32,

    // UI state
    pub drawer_open: bool,
    pub input_buffer: String,
    pub scroll_offset: usize,
    pub selected_command: usize,

    // Runtime state
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub last_tps: f32,
    pub last_response_time: f32,

    // Command registry
    pub command_registry: CommandRegistry,

    // Exit flag
    pub should_exit: bool,

    // Animation/tick state
    tick_count: u32,
}

impl App {
    pub fn new(model: String, temperature: f32) -> Self {
        App {
            session: Session::new(model.clone(), temperature),
            current_model: model,
            temperature,
            drawer_open: false,
            input_buffer: String::new(),
            scroll_offset: 0,
            selected_command: 0,
            is_loading: false,
            error_message: None,
            last_tps: 0.0,
            last_response_time: 0.0,
            command_registry: CommandRegistry::new(),
            should_exit: false,
            tick_count: 0,
        }
    }

    pub fn toggle_drawer(&mut self) {
        self.drawer_open = !self.drawer_open;
        if self.drawer_open {
            self.selected_command = 0;
        }
    }

    pub fn handle_input_char(&mut self, c: char) {
        if self.drawer_open {
            // In drawer mode, arrow keys navigate commands
            return;
        }
        self.input_buffer.push(c);
    }

    pub fn handle_backspace(&mut self) {
        if !self.drawer_open {
            self.input_buffer.pop();
        }
    }

    pub fn submit_input(&mut self) -> Option<String> {
        if self.drawer_open {
            return None;
        }
        let input = self.input_buffer.trim().to_string();
        if !input.is_empty() {
            self.input_buffer.clear();
            return Some(input);
        }
        None
    }

    pub fn scroll_up(&mut self) {
        if !self.drawer_open {
            self.scroll_offset = self.scroll_offset.saturating_add(3);
        } else {
            self.selected_command = self.selected_command.saturating_sub(1);
        }
    }

    pub fn scroll_down(&mut self) {
        if !self.drawer_open {
            self.scroll_offset = self.scroll_offset.saturating_sub(3);
        } else {
            let cmd_count = self.command_registry.get_all_commands().len();
            if self.selected_command < cmd_count.saturating_sub(1) {
                self.selected_command += 1;
            }
        }
    }

    pub fn add_assistant_message(&mut self, text: String, usage: Option<crate::llm_client::Usage>) {
        let tokens = usage.as_ref().map(|u| u.completion_tokens);
        self.session.add_message("assistant".to_string(), text, tokens);
        self.is_loading = false;
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
        self.is_loading = false;
    }


    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    pub fn get_loading_spinner(&self) -> &'static str {
        match self.tick_count % 4 {
            0 => "⠋",
            1 => "⠙",
            2 => "⠹",
            _ => "⠸",
        }
    }

    pub fn get_visible_messages(&self) -> Vec<&crate::session::ChatMessage> {
        self.session.messages.iter().collect()
    }
}
