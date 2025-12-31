use llm::memory;
use async_trait::async_trait;
use crate::{agent::base::BaseAgent, 
            memory::base::BaseMemory, 
            model::{base::BaseModel, litellm_model::Litellm_Model, schema::{LLMResponse, Message, Role}}, 
            prompt::agent::*};

pub struct ReactAgent<M: BaseMemory> {
    model: Litellm_Model,
    system_prompt: Message,
    max_iterations: usize,
    memory: M,
}

impl <M: BaseMemory> ReactAgent<M> {
    pub fn new(model: Litellm_Model, system_prompt: &str, max_iterations: usize, memory: M) -> Self {
        let mut ret = Self {
            model,
            system_prompt: Message::system(system_prompt),
            max_iterations,
            memory,
        };
        ret.system_prompt.content = ret.build_system_prompt(&ret.system_prompt.content);
        ret
    }

    fn build_system_prompt(&self, user_prompt: &str) -> String {
        format!("{}\n\nUser Prompt: {}", REACT_SYSTEM_PROMPT.as_str(), user_prompt)
    }

    fn is_finished(&self, content: &LLMResponse) -> bool {
        match content.content.as_ref() {
            Some(data) => {
                data.contains(REACT_END_TOKEN)
            },
            None => false,
        }
    }

    fn extract_final_answer(&self, content: &LLMResponse) -> Option<String> {
        let content = content.content.as_ref()?;
        if let Some(end_index) = content.find(REACT_END_TOKEN) {
            let answer = &content[..end_index];
            Some(answer.trim().to_string())
        } else {
            None
        }
    }
}

#[async_trait]
impl <M: BaseMemory + Send> BaseAgent for ReactAgent<M> {
   
   async fn add_message(&mut self, message: Message) {
        self.memory.add(message).await;   
   } 
   
   fn build_messages(&self) -> impl Iterator<Item =  &Message> {
        std::iter::once(&self.system_prompt).chain(self.get_history())
   }

   fn clear_history(&mut self) {
        self.memory.clear();
   }

   fn get_history(&self) -> impl Iterator<Item = &Message> {
        self.memory.get_messages()
   }

   async fn run(&mut self, user_prompt: &str) -> String{
        self.add_message(Message::user(user_prompt)).await;
        tracing::debug!("Running ReactAgent with user prompt: {}", user_prompt);

        for i in 0..self.max_iterations {
            let msgs: Vec<&Message> = self.build_messages().collect();
            tracing::debug!("Iteration {}/{}", i + 1, self.max_iterations);
            let response = self.model.call_with_history(msgs).await;
            
            // response is formmatted well for react agent
            if self.is_finished(&response){
                let final_answer = self.extract_final_answer(&response);
                if let Some(answer) = final_answer {
                    self.add_message(Message::assistant(&answer)).await;
                    tracing::debug!("Final answer extracted: {}", answer);
                    return answer;
                }
            }
            // todo: tool calling!!!
        }
        return String::from("Reached maximum iterations without a final answer.");
   }  
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_react_agent_run() {
        let config = crate::config::config::load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        let summary_model = Litellm_Model::new(model_name, model_config.clone(), String::from(""));
        let litellm_model = Litellm_Model::new(model_name, model_config.clone(), String::from(""));
        let memory = crate::memory::summary::SummaryMemory::new("test-1", 0.3, summary_model, 8192, "./workspace_test/");

        let mut agent = ReactAgent::new(
            litellm_model,
            "You are a React Agent. Use tools to answer user queries.",
            3,
            memory,
        );

        let user_prompt = "Can you summarize the plot of 'Inception' and suggest a related movie?";
        let answer = agent.run(user_prompt).await;
        println!("Agent Answer: {}", answer);
    }

    #[tokio::test]
    async fn test_two_react_agents_run() {
        let config = crate::config::config::load_config(None);
        let model_name = "gpt-4o-mini";
        let model_config = config.models.get(model_name).unwrap();
        
        let summary_model = Litellm_Model::new(model_name, model_config.clone(), String::from(""));
        let litellm_model = Litellm_Model::new(model_name, model_config.clone(), String::from(""));
        let memory = crate::memory::summary::SummaryMemory::new("test-1", 0.3, summary_model, 8192, "./workspace_test/");
        let mut agent1 = ReactAgent::new(
            litellm_model,
            "You are a React Agent.",
            3,
            memory,
        );

        let summary_model = Litellm_Model::new(model_name, model_config.clone(), String::from(""));
        let litellm_model = Litellm_Model::new(model_name, model_config.clone(), String::from(""));
        let memory = crate::memory::summary::SummaryMemory::new("test-2", 0.3, summary_model, 8192, "./workspace_test/");
        let mut agent2 = ReactAgent::new(
            litellm_model,
            "You are another React Agent.",
            3,
            memory,
        );

        let user_prompt = "Can you summarize some movies about dreams?";
        let answer = agent1.run(user_prompt).await;
        println!("Agent Answer: {}", answer);
        let follow_up_prompt = format!("Based on the previous answer: {}, can you suggest a movie?", answer);
        let follow_up_answer = agent2.run(&follow_up_prompt).await;
        println!("Follow-up Agent Answer: {}", follow_up_answer);
    }
}