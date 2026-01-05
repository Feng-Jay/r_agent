
use anyhow::Context;
use std::path::PathBuf;
use serde::Deserialize;
use std::fs;
use async_trait::async_trait;
use tiktoken_rs::{get_bpe_from_model, o200k_base};
use crate::{memory::base::BaseMemory, 
            model::{base::BaseModel, litellm_model::LitellmModel, schema::{Message, Role::*}},
            prompt::summary::*,};


/// Summary memory module
/// This module provides a memory implementation that will summarize past interactions when the memory limit is reached.
pub struct SummaryMemory {
    #[allow(dead_code)]
    task_id: String,
    model_str: String,
    reserve_ratio: f32,
    summary_model: LitellmModel,
    max_tokens: usize,
    workspace_path: PathBuf,
    messages: Vec<Message>,
    token_counts: Vec<usize>,
    summary: Message,
    summary_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct Summary {
    pub task_context: String,
    pub key_decisions: Vec<String>,
    pub actions_taken: Vec<String>,
    pub current_state: String,
    pub important_info: Vec<String>,
}

impl SummaryMemory {
    pub fn new(task_id: &str, reserve_ratio: f32, summary_model: LitellmModel, max_tokens: usize, workspace_path: &str) -> Self {
        let mut ret = SummaryMemory {
            task_id: task_id.to_string(),
            model_str: summary_model.model_name.clone(),
            reserve_ratio,
            summary_model: summary_model,
            max_tokens,
            workspace_path: PathBuf::from(workspace_path).join(task_id),
            messages: Vec::new(),
            token_counts: Vec::new(),
            summary: Message{ role: SYSTEM, content: String::new(), tool_calls: None, tool_call_id: None},
            summary_tokens: 0,
        };
        if !ret.workspace_path.exists() {
            fs::create_dir_all(&ret.workspace_path).with_context(|| format!("Failed to create workspace directory: {:?}", &ret.workspace_path)).unwrap();
        }
        ret.load_existing_summary();
        ret
    }

    /// Perform summary of the current messages
    /// keep at least one latest message, and summarize the rest, ensure sum(token_counts) ≤ max_tokens
    async fn do_summary(&mut self) {
        let mut keep_count = 0;
        let mut keep_tokens = 0;
        for tokens in self.token_counts.iter().rev() {
            if keep_tokens + tokens > self.max_tokens{
                break;
            }
            keep_count += 1;
            keep_tokens += tokens;
        }
        keep_count = keep_count.max(1); // at least keep one message
        tracing::debug!("SummaryMemory: keeping last {} messages with {} tokens, summarizing the rest.", keep_count, keep_tokens);
        let to_summarize: Vec<Message> = self.messages.drain(..self.messages.len() - keep_count).collect();
        self.token_counts.drain(..self.token_counts.len() - keep_count);
        if to_summarize.len() == 0 {
            // current summarization already exceed the limit, need to compress the existing summary
            if self.summary.content.len() > 0 && self.summary_tokens > self.summary_budget() {
                self.compress_summary().await;
            }
            self.save_summary();
            return;
        }
        
        let conversation_str = self.format_conversation(to_summarize);
        
        let prompt_message = Message{role: USER, 
                                              content: SUMMARY_PROMPT.replace("{conversation}", &conversation_str),
                                              tool_calls: None,
                                              tool_call_id: None,
                                             };
        let mut retry = 0;
        let max_retries = 3;
        let summary_text = loop {
            let response = self.summary_model.call(&prompt_message).await;
            match response.content {
                Some(text) => {
                    match self.parse_summary(&text){
                        Ok(summary) => {
                            tracing::info!("Generated well formatted summary.");
                            break SUMMARY_FORMAT.replace("{task_context}", &summary.task_context)
                                                            .replace("{key_decisions}", &summary.key_decisions.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n"))
                                                            .replace("{actions_taken}", &summary.actions_taken.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n"))
                                                            .replace("{current_state}", &summary.current_state)
                                                            .replace("{important_info}", &summary.important_info.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n"));
                        },
                        Err(e) => {
                            tracing::error!("Failed to parse summary JSON: {}, retrying times: {}...", e, retry);
                            retry += 1;
                            if retry >= max_retries {
                                tracing::error!("Failed to get summary after 3 retries. Use first 1000 chars as summary.");
                                break text.chars().take(text.len().min(1000)).collect();
                            }
                        }
                    }
                },
                None => {
                    tracing::error!("Summary model returned no content, retrying times: {}...", retry);
                    retry += 1;
                    if retry >= max_retries {
                        tracing::error!("Failed to get summary after 3 retries. Use half of original conversation as summary.");
                        break self.summary.content.chars().take(self.summary.content.len()/2).collect();
                   }
                }
            }
        };

        let bpe = get_bpe_from_model(&self.model_str).unwrap_or(o200k_base().unwrap());
        self.summary.content = if self.summary.content.len() > 0 {format!("{}\n\n---\n\n{}", self.summary.content, summary_text)} else {summary_text};
        self.summary_tokens = bpe.encode_with_special_tokens(self.summary.content.as_str()).len();

        if self.summary_tokens > self.summary_budget() {
            self.compress_summary().await;
        }

        self.save_summary();

    }

    fn parse_summary(&self, summary_text: &str) -> Result<Summary, serde_json::Error>{
        let start = summary_text.find("{").unwrap_or(0);
        let end = summary_text.rfind("}").unwrap_or(summary_text.len()-1);
        let json_str = &summary_text[start..=end];
        let v= serde_json::from_str::<Summary>(json_str).map_err(|e| {
            tracing::error!("Failed to parse summary JSON: {} from json_str:\n {}", e, json_str);
            e
        });
        v
    }

    async fn compress_summary(&mut self) {
        let prompt = COMPRESS_SUMMARY_PROMPT.replace("{target_tokens}", &self.summary_budget().to_string())
                                                    .replace("{summary}", &self.summary.content);
        let prompt_message = Message{role: USER, 
                                              content: prompt,
                                              tool_calls: None,
                                              tool_call_id: None,
                                             };
        let mut retry = 0;
        let max_retries = 3;
        let compressed_summary = loop {
            let response = self.summary_model.call(&prompt_message).await;
            match response.content {
                Some(text) => {
                    match self.parse_summary(&text){
                        Ok(summary) => {
                            tracing::info!("Compressed well formatted summary.");
                            break SUMMARY_FORMAT.replace("{task_context}", &summary.task_context)
                                                            .replace("{key_decisions}", &summary.key_decisions.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n"))
                                                            .replace("{actions_taken}", &summary.actions_taken.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n"))
                                                            .replace("{current_state}", &summary.current_state)
                                                            .replace("{important_info}", &summary.important_info.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n"));
                        },
                        Err(e) => {
                            tracing::error!("Failed to parse compressed summary JSON: {}, retrying times: {}...", e, retry);
                            retry += 1;
                            if retry >= max_retries {
                                tracing::error!("Failed to compress summary after 3 retries. Use first 1000 chars as summary.");
                                break text.chars().take(text.len().min(1000)).collect();
                            }
                        }
                    }
                },
                None => {
                    tracing::error!("Summary compression model returned no content, retrying times: {}...", retry);
                    retry += 1;
                    if retry >= max_retries {
                        tracing::error!("Failed to compress summary after 3 retries. Use half of original summary as compressed summary.");
                        break self.summary.content.chars().take(self.summary.content.len()/2).collect();
                   }
                }
            }
        };
        let bpe = get_bpe_from_model(&self.model_str).unwrap_or(o200k_base().unwrap());
        self.summary.content = compressed_summary;
        self.summary_tokens = bpe.encode_with_special_tokens(&self.summary.content.as_str()).len();
    }    

    // following are private helper/getter functions
    fn summary_file(&self) -> PathBuf {
        self.workspace_path.join("summary.txt")
    }

    fn reserve_tokens(&self) -> usize {
        (self.max_tokens as f32 * self.reserve_ratio) as usize
    }

    fn summary_budget(&self) -> usize {
        self.max_tokens - self.reserve_tokens()
    }

    fn load_existing_summary(&mut self){
        let summary_path = self.summary_file();   
        if summary_path.exists() {
            if let Ok(content) = fs::read_to_string(&summary_path) {
                let bpe = get_bpe_from_model(&self.model_str).unwrap_or(o200k_base().unwrap());
                self.summary.content = format!("Previous conversation summary:\n {}", content);
                self.summary_tokens = bpe.encode_with_special_tokens(self.summary.content.as_str()).len();
            }
        }
    }

    fn save_summary(&self){
        let summary_path = self.summary_file();   
        if let Err(e) = fs::write(&summary_path, &self.summary.content) {
            tracing::error!("Failed to save summary to file {:?}: {}", summary_path, e);
        }
    }
    
    fn format_conversation(&self, messages: Vec<Message>) -> String {
        let mut ret = String::new();
        for msg in messages.into_iter() {
            let role = msg.role.to_string();
            let mut content = {
                let mut s = msg.content; // move String out
                if s.chars().count() > 500 {
                    s = s.chars().take(500).collect::<String>() + "...[truncated]";
                }
                s
            };
            let tool_calls = match msg.tool_calls {
                Some(calls) => format!("With Tool Calls: {:?}", calls),
                None => "".to_string(),
            };
            let tool_call_id = match msg.tool_call_id {
                Some(id) => {content = format!("result: {}", content);format!("With Tool Call ID: {}", id) },
                None => "".to_string(),
            };
            ret.push_str(&format!("{}: {} {} {}", role, content, tool_calls, tool_call_id));
        }
        ret
    }

}

#[async_trait]
impl BaseMemory for SummaryMemory {
    async fn add(&mut self, message: Message) {
        let bpe = get_bpe_from_model(&self.model_str).unwrap_or(o200k_base().unwrap());
        self.token_counts.push(bpe.encode_with_special_tokens(message.content.as_str()).len());
        self.messages.push(message);
        tracing::debug!("Added message to SummaryMemory, current token count: {}", self.token_count());
        if self.token_count() > self.max_tokens {
            self.do_summary().await;
        }
    }

    fn get_messages(&self) -> impl Iterator<Item = &Message> {
        std::iter::once(&self.summary).chain(self.messages.iter())
    }

    fn token_count(&self) -> usize {
        self.token_counts.iter().sum()        
    }

    fn clear(&mut self) {
        self.messages.clear();
        self.token_counts.clear();        
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_summary_memory() {
        let config = crate::config::config::load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let summary_model = LitellmModel::new(model_name, model_config.clone(), "");

        let mut memory = SummaryMemory::new("test_task", 0.2, summary_model, 100, "./workspace");
        for i in 0..15 {
            let content = format!("This is test message number {}. {}", i, "A".repeat(50));
            memory.add(Message::user(&content)).await;
        }
        let total_tokens = memory.token_count();
        assert!(total_tokens <= 100);
        let msgs: Vec<&Message> = memory.get_messages().collect();
        assert!(msgs.len() >= 1); // at least the summary message should be there
        println!("Messages in memory:");
        for msg in msgs {
            println!("{}: {}", msg.role.to_string(), msg.content);
        }
    }
}