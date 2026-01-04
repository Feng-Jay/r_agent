use serde_json::{json, Value};
use tracing::debug;
use anyhow::Context;
use async_trait::async_trait;
use llm::{FunctionCall, LLMProvider, ToolCall, builder::{FunctionBuilder, LLMBackend, LLMBuilder, ParamBuilder}, chat::ChatMessage};
use crate::{config::config::{ModelConfig}, 
            model::{base::BaseModel},
            model::schema::{LLMResponse, Message, Role, Usage}};


pub struct Litellm_Model{
    pub model_name: String,
    config: ModelConfig,
    llm: Box<dyn LLMProvider>
}

impl Litellm_Model {
    pub fn new(model_name:&str, settings: ModelConfig, system_prompt: &str) -> Self {
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
        if let Some(cost) = &settings.cost{
            llm_builder = llm_builder.max_tokens(cost.max_tokens as u32);
        }
        if let Some(temperature) = settings.temperature {
            llm_builder = llm_builder.temperature(temperature);
        }
        if let Some(top_p) = settings.top_p {
            llm_builder = llm_builder.top_p(top_p);
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

    pub fn new_with_tools(model_name:&str, settings: ModelConfig, system_prompt: &str, functions: Vec<Value>) -> Self {
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
        if let Some(cost) = &settings.cost{
            llm_builder = llm_builder.max_tokens(cost.max_tokens as u32);
        }
        if let Some(temperature) = settings.temperature {
            llm_builder = llm_builder.temperature(temperature);
        }
        if let Some(top_p) = settings.top_p {
            llm_builder = llm_builder.top_p(top_p);
        }
        
        tracing::debug!("Adding functions to LLM: {:?}", functions);
        for func in functions {
            let func_name = func.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let func_description = func.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let parameters = func.get("parameters").cloned().unwrap_or(Value::Null);
            let function_builder = FunctionBuilder::new(func_name).description(func_description).json_schema(parameters);
            llm_builder = llm_builder.function(function_builder);
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

    fn build_message(&self, msg: &Message) -> ChatMessage {
        match msg.role {
            Role::ASSISTANT => {
                if let Some(tool_calls) = msg.tool_calls.as_ref() {
                    ChatMessage::assistant()
                        .tool_use(tool_calls.clone())
                        .build()
                } else {
                    ChatMessage::assistant()
                        .content(msg.content.clone())
                        .build()
                }
            }
            Role::TOOL => {
                let id = msg.tool_call_id.as_ref().unwrap().clone();
                let tool_call_res = ToolCall{
                    id,
                    call_type: "function".into(),
                    function: FunctionCall{
                        name: msg.tool_calls.as_ref().unwrap()[0].function.name.clone(),
                        arguments: json!({"status": "ok", "results": msg.content}).to_string(),
                    }
                };
                ChatMessage::assistant().tool_result(vec![tool_call_res]).build()
            }
            _ => ChatMessage::user().content(msg.content.clone()).build(),
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
                    },
                    tool_calls: response.tool_calls()
                }
            },
            Err(e) => {
                tracing::error!("Error during LLM call: {}", e);
                LLMResponse { 
                    content: None,
                    reasoning_content: None,
                    usage: None,
                    tool_calls: None
                }
            }
        }
    }
}

#[async_trait]
impl BaseModel for Litellm_Model {
    async fn call(&self, user_prompt: &Message) -> LLMResponse {
        let mut history = Vec::new();
        history.push(user_prompt);
        self.call_with_history(history).await
    }

    async fn call_with_history(
            &self,
            history: Vec<&Message>,
        ) -> LLMResponse {
        let mut messages = Vec::new();
        for msg in history {
            let chat_msg = self.build_message(&msg);
            messages.push(chat_msg);
        }
        // let user_msg = self.build_message(&Role::USER, user_prompt);
        // messages.push(user_msg);
        let llm_response = self._do_call(&messages).await;
        llm_response
    }
}


// ------------------ Unit Test Module ------------------
#[cfg(test)]
mod tests {
    use std::{result, vec};

    use super::*;
    use crate::{config::config::load_config, tool};
    
    #[tokio::test]
    async fn test_llm() {
        let config = load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let litellm_model = Litellm_Model::new(model_name, model_config.clone(), "");
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
        let litellm_model = Litellm_Model::new(model_name, model_config.clone(), "");
        let user_prompt = "Hello, how about the weather of NY today";
        let out = litellm_model.call(&Message::user(user_prompt)).await;
        println!("\nOutput: {:?}", out);
    }

    #[tokio::test]
    async fn test_llm_with_call_with_history(){
        let config = load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let litellm_model = Litellm_Model::new(model_name, model_config.clone(), "");
        let history = vec![
            Message::user("Hello, how about the weather of NY today"),
            Message::assistant("The weather in NY today is sunny with a high of 75°F.", None),
            Message::user("Can you give me a summary of our previous conversation?")
        ];
        let history_refs: Vec<&Message> = history.iter().collect();
        let out = litellm_model.call_with_history(history_refs).await;
        println!("\nOutput: {:?}", out);
    }

    #[tokio::test]
    async fn test_llm_with_tools(){
        let config = load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let functions = vec![
            json!({
                "name": "get_current_weather",
                "description": "Get the current weather in a given location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city and state, e.g. San Francisco, CA"
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["celsius", "fahrenheit"]
                        }
                    },
                    "required": ["location"]
                }
            })
        ];
        let litellm_model = Litellm_Model::new_with_tools(model_name, model_config.clone(), "", functions);
        let user_prompt = "What's the weather like in Boston?";
        let out = litellm_model.call(&Message::user(user_prompt)).await;
        let output = out.content.unwrap_or("No content".to_string());
        let tool_calls = out.tool_calls.unwrap();
        let result = "current weather in Boston is 68°F and sunny";
        let prompts = vec![
            Message::user("What's the weather like in Boston?"),
            Message::assistant(&output, Some(tool_calls.clone())),
            Message::tool(result, Some(tool_calls.clone()), Some(tool_calls[0].id.clone()))
        ];
        let prompt_refs: Vec<&Message> = prompts.iter().collect();
        let out_with_history = litellm_model.call_with_history(prompt_refs).await;
        println!("\nOutput with tools: {:?}", out_with_history);
    }

    #[tokio::test]
    async fn test_llm_with_tools2(){
        let config = load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let functions = vec![
            json!({
                "name": "calculate_sum",
                "description": "Calculate the sum of two numbers",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "num1": {
                            "type": "number",
                            "description": "The first number"
                        },
                        "num2": {
                            "type": "number",
                            "description": "The second number"
                        }
                    },
                    "required": ["num1", "num2"]
                }
            })
        ];
        let litellm_model = Litellm_Model::new_with_tools(model_name, model_config.clone(), "", functions);
        let user_prompt = "What's the sum of 1000 and 10000?";
        tracing::debug!("User prompt: {}", user_prompt);
        let out = litellm_model.call(&Message::user(user_prompt)).await;
        let output = out.content.unwrap_or("No content".to_string());
        let tool_calls = out.tool_calls.unwrap();
        let result = "11000";
        let prompts = vec![
            Message::user(user_prompt),
            Message::assistant(&output, Some(tool_calls.clone())),
            Message::tool(result, Some(tool_calls.clone()), Some(tool_calls[0].id.clone()))
        ];
        let prompt_refs: Vec<&Message> = prompts.iter().collect();
        let out_with_history = litellm_model.call_with_history(prompt_refs).await;
        println!("\nOutput with tools: {:?}", out_with_history);
    }
}