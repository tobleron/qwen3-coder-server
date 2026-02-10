use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Local};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: usize,              // Sequence ID within session
    pub role: String,           // "user" or "assistant"
    pub content: String,        // Message text
    pub timestamp: DateTime<Utc>,
    #[allow(dead_code)]
    pub tokens: Option<u32>,    // Token count if available
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionMetadata {
    pub id: String,             // session_DDMMYYYY_HHMMSS[_label]
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub model_name: String,
    pub temperature: f32,
    pub message_count: usize,
    #[allow(dead_code)]
    pub total_tokens: u32,
    pub label: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub metadata: SessionMetadata,
    pub messages: Vec<ChatMessage>,
}

impl Session {
    pub fn new(model_name: String, temperature: f32) -> Self {
        let timestamp = Local::now().format("%d%m%Y_%H%M%S").to_string();
        let id = format!("session_{}", timestamp);

        Session {
            metadata: SessionMetadata {
                id: id.clone(),
                created_at: Utc::now(),
                last_modified: Utc::now(),
                model_name,
                temperature,
                message_count: 0,
                total_tokens: 0,
                label: None,
            },
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, role: String, content: String, tokens: Option<u32>) {
        let id = self.messages.len() + 1;
        self.messages.push(ChatMessage {
            id,
            role,
            content,
            timestamp: Utc::now(),
            tokens,
        });
        self.metadata.message_count = self.messages.len();
        self.metadata.last_modified = Utc::now();
        if let Some(t) = tokens {
            self.metadata.total_tokens += t;
        }
    }

    pub fn save(&self, base_dir: &str) -> anyhow::Result<()> {
        let session_dir = Path::new(base_dir).join(&self.metadata.id);
        fs::create_dir_all(&session_dir)?;

        let metadata_path = session_dir.join("metadata.json");
        let messages_path = session_dir.join("messages.json");

        fs::write(metadata_path, serde_json::to_string_pretty(&self.metadata)?)?;
        fs::write(messages_path, serde_json::to_string_pretty(&self.messages)?)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn load(base_dir: &str, session_id: &str) -> anyhow::Result<Self> {
        let session_dir = Path::new(base_dir).join(session_id);

        let metadata: SessionMetadata = serde_json::from_str(
            &fs::read_to_string(session_dir.join("metadata.json"))?
        )?;

        let messages: Vec<ChatMessage> = serde_json::from_str(
            &fs::read_to_string(session_dir.join("messages.json"))?
        )?;

        Ok(Session { metadata, messages })
    }

    #[allow(dead_code)]
    pub fn list_sessions(base_dir: &str) -> anyhow::Result<Vec<SessionMetadata>> {
        let mut sessions = Vec::new();

        if !Path::new(base_dir).exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(base_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let metadata_path = entry.path().join("metadata.json");
                if metadata_path.exists() {
                    let metadata: SessionMetadata = serde_json::from_str(
                        &fs::read_to_string(metadata_path)?
                    )?;
                    sessions.push(metadata);
                }
            }
        }

        // Sort by last modified, newest first
        sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        Ok(sessions)
    }

    #[allow(dead_code)]
    pub fn delete_session(base_dir: &str, session_id: &str) -> anyhow::Result<()> {
        let session_dir = Path::new(base_dir).join(session_id);
        fs::remove_dir_all(session_dir)?;
        Ok(())
    }

    pub fn rename(&mut self, label: String) {
        self.metadata.label = Some(label.clone());
        // Update ID to include label
        let base_parts: Vec<&str> = self.metadata.id.split('_').collect();
        if base_parts.len() >= 3 {
            let base_id = format!("{}_{}", base_parts[0], base_parts[1]);
            self.metadata.id = format!("{}_{}", base_id, label);
        }
        self.metadata.last_modified = Utc::now();
    }

    pub fn delete_message(&mut self, id: usize) -> anyhow::Result<()> {
        self.messages.retain(|m| m.id != id);
        self.metadata.message_count = self.messages.len();
        self.metadata.last_modified = Utc::now();
        Ok(())
    }

    pub fn clear_all(&mut self) {
        self.messages.clear();
        self.metadata.message_count = 0;
        self.metadata.total_tokens = 0;
        self.metadata.last_modified = Utc::now();
    }

    pub fn get_message(&self, id: usize) -> Option<&ChatMessage> {
        self.messages.iter().find(|m| m.id == id)
    }
}
