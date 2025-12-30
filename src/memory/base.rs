use async_trait::async_trait;
use crate::model::schema::Message;

#[async_trait]
pub trait BaseMemory {
    // Add a message to the memory
    async fn add(&mut self, message:Message);
    // Get all messages from the memory
    fn get_messages(&self) -> impl Iterator<Item = &Message>;
    // Clear the memory
    fn clear(&mut self);
    // Get the token count of the memory
    fn token_count(&self) -> usize;
}