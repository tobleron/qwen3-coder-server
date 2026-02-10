use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::RuboxConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone)]
pub struct LlmClient {
    api_url: String,
    model_name: String,
    client: Client,
    #[allow(dead_code)]
    pub context_window: u32,
    pub temperature: f32,
}

#[derive(Serialize)]
struct CompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct CompletionResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize, Debug)]
struct MessageContent {
    content: String,
}

impl LlmClient {
    pub fn new(config: &RuboxConfig) -> Self {
        LlmClient {
            api_url: config.llm.api_url.clone(),
            model_name: config.llm.model_name.clone(),
            client: Client::new(),
            context_window: config.llm.context_window,
            temperature: config.temperature.default,
        }
    }

    #[allow(dead_code)]
    pub async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<String, reqwest::Error> {
        self.chat_completion_with_usage(messages, self.temperature).await.map(|(content, _)| content)
    }

    pub async fn chat_completion_with_usage(&self, messages: Vec<ChatMessage>, temperature: f32) -> Result<(String, Option<Usage>), reqwest::Error> {
        let url = format!("{}/chat/completions", self.api_url);

        let request = CompletionRequest {
            model: self.model_name.clone(),
            messages,
            temperature,
            max_tokens: 4096,
        };

        let res = self.client.post(url)
            .json(&request)
            .send()
            .await?;

        let response_data: CompletionResponse = res.json().await?;
        let content = response_data.choices[0].message.content.clone();
        Ok((content, response_data.usage))
    }
}
