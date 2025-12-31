use crate::model::litellm_model::Litellm_Model;
use crate::model::base::BaseModel;
use crate::model::schema::*;
use crate::memory::{base::BaseMemory};
use async_trait::async_trait;

#[async_trait]
pub trait BaseAgent {
    fn get_history(&self) -> impl Iterator<Item = &Message>;
    fn clear_history(&mut self);
    fn build_messages(&self) -> impl Iterator<Item = &Message>;
    async fn add_message(&mut self, message: Message);
    async fn run(&mut self, user_prompt: &str) -> String;
}