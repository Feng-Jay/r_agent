use crate::memory::base::BaseMemory;
use crate::model::schema::Message;
use tiktoken_rs::{get_bpe_from_model, o200k_base};
use async_trait::async_trait;

pub struct SlidingWindowMemory {
    max_messages: usize,
    max_tokens: usize,
    model_str: String,
    messages: Vec<Message>,
    token_counts: Vec<usize>,
}

impl SlidingWindowMemory {
    pub fn new(max_messages: usize, model: &str, max_tokens: usize) -> Self {
        SlidingWindowMemory {
            max_messages,
            max_tokens,
            model_str: model.to_string(),
            messages: Vec::new(),
            token_counts: Vec::new(),
        }
    }

    fn _truncate(&mut self) {
        if self.messages.len() > self.max_messages {
            let exces = self.messages.len() - self.max_messages;
            self.messages.drain(..exces);
            self.token_counts.drain(..exces);
        }
        while self.token_count() > self.max_tokens {
            self.messages.remove(0);
            self.token_counts.remove(0);
        }
    }
}

#[async_trait]
impl BaseMemory for SlidingWindowMemory {
    async fn add(&mut self, message: Message) {
        let bpe = get_bpe_from_model(&self.model_str).unwrap_or(o200k_base().unwrap());
        self.token_counts.push(bpe.encode_with_special_tokens(message.content.as_str()).len());
        self.messages.push(message);
        self._truncate();
    }

    fn get_messages(&self) -> impl Iterator<Item = &Message> {
        self.messages.iter()
    }

    fn token_count(&self) -> usize {
        self.token_counts.iter().sum()       
    }
    
    fn clear(&mut self) {
        self.messages.clear();
        self.token_counts.clear();    
    }
}


// ------------------ Unit Test Module ------------------
#[cfg(test)]
mod tests{
    use super::*;
    use crate::model::schema::Message;

    #[test]
    fn test_sliding_window_memory() {
        let mut memory = SlidingWindowMemory::new(3, "gpt-4o-mini", 20);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            memory.add(Message::user("Hello")).await;
            memory.add(Message::user("How are you?")).await;
            memory.add(Message::user("Tell me a joke.")).await;
            let msgs: Vec<&Message> = memory.get_messages().collect();
            assert_eq!(msgs.len(), 3);
            memory.add(Message::user("What's the weather like?")).await;
            let more: Vec<&Message> = memory.get_messages().collect();
            assert_eq!(more.len(), 3);
            assert_eq!(more[0].content, "How are you?")
        });
    }
}