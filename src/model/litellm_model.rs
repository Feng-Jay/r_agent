use crate::{config::config::{Config, ModelConfig}, model::base::BaseModel};
use anyhow::Context;
use async_trait::async_trait;
use llm::{
    LLMProvider, builder::{LLMBackend, LLMBuilder}, chat::ChatMessage
};

pub struct Litellm_Model{
    model_name: String,
    config: ModelConfig,
    llm: Box<dyn LLMProvider>
}

impl Litellm_Model {
    pub fn new(model_name:&str, settings: ModelConfig) -> Self {
        let api_key = settings.api_key.as_str();

        let llm_builder = LLMBuilder::new()
                                .backend(LLMBackend::OpenAI)
                                .api_key(api_key)
                                .model(model_name);

        let llm_builder = match settings.base_url.as_deref(){
            Some(base_url) => llm_builder.base_url(base_url),
            None => llm_builder,
        };
        
        let llm = llm_builder
                                .build()
                                .with_context(|| format!("Failed to build LLM model: {}", model_name)).unwrap();
        Litellm_Model {
            model_name: model_name.to_string(),
            config: settings,
            llm: llm
        }
    }

    pub async fn _do_call(&self, messages: &Vec<ChatMessage>) -> String {
        match self.llm.chat(messages).await {
            Ok(response) => {
                println!("LLM Response: {:?}", response.text());
                println!("Usage: {:?}", response.usage());
                response.text().unwrap()
            },
            Err(e) => {
                eprintln!("Error during LLM call: {}", e);
                "".to_string()
            }
        }
    }
}

#[async_trait]
impl BaseModel for Litellm_Model {
    async fn call(&self, user_prompt: &str, system_prompt: Option<&str>) -> String {
     "".to_string()
    }

    async fn call_with_history(
            &self,
            user_prompt: &str,
            history: Vec<&str>,
            system_prompt: Option<&str>,
        ) -> String {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::config::load_config, model::{self, litellm_model}};

    use super::*;
    #[tokio::test]
    async fn test_llm() {
        let config = load_config(None);
        println!("{:?}", config);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let litellm_model = Litellm_Model::new(model_name, model_config.clone());
        let messages = vec![
            ChatMessage::user().content("You are a helpful assistant.").build(),
            ChatMessage::user().content("Hello, how are you?").build(),
        ];
        let out = litellm_model._do_call(&messages).await;
        println!("\nOutput: {:?}", out);
        assert!(!out.is_empty());
    }
}