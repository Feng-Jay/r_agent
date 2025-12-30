use tracing::debug;
use anyhow::Context;
use async_trait::async_trait;
use llm::{LLMProvider, 
          builder::{LLMBackend, LLMBuilder, FunctionBuilder, ParamBuilder}, 
          chat::ChatMessage};
use crate::{config::config::{Config, ModelConfig}, 
            model::{base::BaseModel},
            model::schema::{LLMResponse, Message, Role, Usage}};


pub struct Litellm_Model{
    model_name: String,
    config: ModelConfig,
    llm: Box<dyn LLMProvider>
}

impl Litellm_Model {
    pub fn new(model_name:&str, settings: ModelConfig, system_prompt: String) -> Self {
        let api_key = settings.api_key.as_str();

        let mut llm_builder = LLMBuilder::new()
                                .backend(LLMBackend::OpenAI)
                                .api_key(api_key)
                                .model(model_name);
        
        if system_prompt.len() > 0 {
            llm_builder = llm_builder.system(system_prompt);
        }

        if let Some(base_url) = settings.base_url.as_deref() {
            llm_builder = llm_builder.base_url(base_url);
        }
        
        let llm = llm_builder
                                .build()
                                .with_context(|| format!("Failed to build LLM model: {}", model_name)).unwrap();
        Litellm_Model {
            model_name: model_name.to_string(),
            config: settings,
            llm: llm
        }
    }

    fn build_message(&self, role: &Role, content: &str) -> ChatMessage {
        match role {
            Role::ASSISTANT => ChatMessage::assistant().content(content).build(),
            _ => ChatMessage::user().content(content).build(),
        }
    }

    pub async fn _do_call(&self, messages: &Vec<ChatMessage>) -> LLMResponse {
        match self.llm.chat(messages).await {
            Ok(response) => {
                tracing::debug!("LLM Response: {:?}", response.text());
                tracing::debug!("Usage: {:?}", response.usage());
                tracing::debug!("tool_calls: {:?}", response.tool_calls());
                LLMResponse {
                    content: response.text(),
                    reasoning_content: response.thinking(),
                    usage: if let Some(usage) = response.usage() {
                        Some(Usage {
                            prompt_tokens: usage.prompt_tokens,
                            completion_tokens: usage.completion_tokens,
                            total_tokens: usage.total_tokens,
                        })
                    } else {
                        None
                    }
                }
            },
            Err(e) => {
                tracing::error!("Error during LLM call: {}", e);
                LLMResponse { 
                    content: None,
                    reasoning_content: None,
                    usage: None
                }
            }
        }
    }
}

#[async_trait]
impl BaseModel for Litellm_Model {
    async fn call(&self, user_prompt: &Message) -> LLMResponse {
        let history = Vec::new();
        self.call_with_history(&user_prompt.content, &history).await
    }

    async fn call_with_history(
            &self,
            user_prompt: &str,
            history: &Vec<Message>,
        ) -> LLMResponse {
        let mut messages = Vec::new();
        for msg in history {
            let chat_msg = self.build_message(&msg.role, &msg.content);
            messages.push(chat_msg);
        }
        let user_msg = self.build_message(&Role::USER, user_prompt);
        messages.push(user_msg);
        let llm_response = self._do_call(&messages).await;
        llm_response
    }
}


// ------------------ Unit Test Module ------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config::load_config;
    
    #[tokio::test]
    async fn test_llm() {
        let config = load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let litellm_model = Litellm_Model::new(model_name, model_config.clone(), String::from(""));
        let messages = vec![
            ChatMessage::user().content("You are a helpful assistant.").build(),
            ChatMessage::user().content("Hello, how about the weather of NY today").build(),
        ];
        let out = litellm_model._do_call(&messages).await;
        println!("\nOutput: {:?}", out);
    }

    #[tokio::test]
    async fn test_llm_with_call(){
        let config = load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let litellm_model = Litellm_Model::new(model_name, model_config.clone(), String::from(""));
        let user_prompt = "Hello, how about the weather of NY today";
        let out = litellm_model.call(&Message::user(user_prompt)).await;
        println!("\nOutput: {:?}", out);
    }

    #[tokio::test]
    async fn test_llm_with_call_with_history(){
        let config = load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let litellm_model = Litellm_Model::new(model_name, model_config.clone(), String::from(""));
        let user_prompt = "Can you give me a summary of our previous conversation?";
        let history = vec![
            Message::user("Hello, how about the weather of NY today"),
            Message::assistant("The weather in NY today is sunny with a high of 75°F."),
        ];
        let out = litellm_model.call_with_history(user_prompt, &history).await;
        println!("\nOutput: {:?}", out);
    }
}