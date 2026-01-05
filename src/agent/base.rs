use crate::model::schema::*;
use async_trait::async_trait;

#[async_trait]
pub trait BaseAgent {
    fn get_history(&self) -> impl Iterator<Item = &Message>;
    fn clear_history(&mut self);
    fn build_messages(&self) -> impl Iterator<Item = &Message>;
    async fn add_message(&mut self, message: Message);
    async fn run(&mut self, user_prompt: &str) -> String;
}