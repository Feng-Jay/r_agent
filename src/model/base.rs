use async_trait::async_trait;
use crate::model::schema::{LLMResponse, Message};

#[async_trait]
pub trait BaseModel {
    async fn call(&self, user_prompt: &Message) -> LLMResponse;
    async fn call_with_history(
        &self,
        history: Vec<&Message>
    ) -> LLMResponse;
}