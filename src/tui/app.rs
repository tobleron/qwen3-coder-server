use crate::session::Session;
use crate::commands::CommandRegistry;

#[derive(Debug, Clone, PartialEq)]
pub enum UIMode {
    Chat,
    CommandPalette,
    Modal(ModalType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalType {
    ModelSelector,
    SetTemperature,
    DeleteMessage,
    SaveResponse,
    RenameSession,
    LoadPrompt,
}

pub struct App {
    // Session data
    pub session: Session,
    pub current_model: String,
    pub temperature: f32,

    // UI state
    pub mode: UIMode,
    pub input_buffer: String,
    pub command_search: String,
    pub scroll_offset: usize,
    pub selected_command_idx: usize,
    pub modal_input: String,

    // Runtime state
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub last_tps: f32,
    pub last_response_time: f32,

    // Command registry
    pub command_registry: CommandRegistry,

    // Model registry (name -> full filename path)
    pub model_registry: std::collections::HashMap<String, String>,

    // Exit flag
    pub should_exit: bool,

    // Animation/tick state
    tick_count: u32,
}

impl App {
    pub fn new(
        model: String,
        temperature: f32,
        model_registry: std::collections::HashMap<String, String>,
    ) -> Self {
        App {
            session: Session::new(model.clone(), temperature),
            current_model: model,
            temperature,
            mode: UIMode::Chat,
            input_buffer: String::new(),
            command_search: String::new(),
            scroll_offset: 0,
            selected_command_idx: 0,
            modal_input: String::new(),
            is_loading: false,
            error_message: None,
            last_tps: 0.0,
            last_response_time: 0.0,
            command_registry: CommandRegistry::new(),
            model_registry,
            should_exit: false,
            tick_count: 0,
        }
    }

    pub fn open_command_palette(&mut self) {
        self.mode = UIMode::CommandPalette;
        self.command_search.clear();
        self.selected_command_idx = 0;
    }

    pub fn close_command_palette(&mut self) {
        self.mode = UIMode::Chat;
        self.command_search.clear();
    }

    pub fn open_modal(&mut self, modal: ModalType) {
        self.mode = UIMode::Modal(modal);
        self.modal_input.clear();
    }

    pub fn close_modal(&mut self) {
        self.mode = UIMode::Chat;
        self.modal_input.clear();
    }

    pub fn handle_input_char(&mut self, c: char) {
        match self.mode {
            UIMode::Chat => {
                self.input_buffer.push(c);
            }
            UIMode::CommandPalette => {
                self.command_search.push(c);
                self.selected_command_idx = 0; // Reset selection when searching
            }
            UIMode::Modal(_) => {
                self.modal_input.push(c);
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.mode {
            UIMode::Chat => {
                self.input_buffer.pop();
            }
            UIMode::CommandPalette => {
                self.command_search.pop();
            }
            UIMode::Modal(_) => {
                self.modal_input.pop();
            }
        }
    }

    pub fn submit_input(&mut self) -> Option<String> {
        match self.mode {
            UIMode::Chat => {
                let input = self.input_buffer.trim().to_string();
                if !input.is_empty() {
                    self.input_buffer.clear();
                    return Some(input);
                }
            }
            UIMode::CommandPalette => {
                // User is selecting a command
                let filtered = self.get_filtered_commands();
                if self.selected_command_idx < filtered.len() {
                    let cmd_name = filtered[self.selected_command_idx].name;
                    self.close_command_palette();
                    return Some(format!("/{}", cmd_name));
                }
            }
            UIMode::Modal(_) => {
                let input = self.modal_input.trim().to_string();
                self.modal_input.clear();
                if !input.is_empty() {
                    self.close_modal();
                    return Some(input);
                }
            }
        }
        None
    }

    pub fn get_filtered_commands(&self) -> Vec<&crate::commands::Command> {
        if self.command_search.is_empty() {
            self.command_registry.get_all_commands().iter().collect()
        } else {
            self.command_registry
                .get_all_commands()
                .iter()
                .filter(|cmd| {
                    cmd.name.contains(&self.command_search)
                        || cmd.help.contains(&self.command_search)
                })
                .collect()
        }
    }

    pub fn scroll_up(&mut self) {
        match self.mode {
            UIMode::Chat => {
                self.scroll_offset = self.scroll_offset.saturating_add(3);
            }
            UIMode::CommandPalette => {
                self.selected_command_idx = self.selected_command_idx.saturating_sub(1);
            }
            UIMode::Modal(_) => {
                // No scrolling in modal
            }
        }
    }

    pub fn scroll_down(&mut self) {
        match self.mode {
            UIMode::Chat => {
                self.scroll_offset = self.scroll_offset.saturating_sub(3);
            }
            UIMode::CommandPalette => {
                let count = self.get_filtered_commands().len();
                if self.selected_command_idx < count.saturating_sub(1) {
                    self.selected_command_idx += 1;
                }
            }
            UIMode::Modal(_) => {
                // No scrolling in modal
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
