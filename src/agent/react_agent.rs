use anyhow::Context;
use llm::ToolCall;
use serde_json::Value;
use async_trait::async_trait;
use crate::{agent::{base::BaseAgent, tool_agent::ToolAgent}, 
            config::config::Config, memory::base::BaseMemory, 
            model::{base::BaseModel, litellm_model::LitellmModel, schema::{LLMResponse, Message}},
            prompt::agent::*, 
            tool::manager::ToolManager};


 pub struct ReactAgent<M: BaseMemory> {
    model: LitellmModel,
    system_prompt: Message,
    max_iterations: usize,
    tool_manager: ToolManager,
    tool_names: Vec<String>,
    memory: M,
}


impl <M: BaseMemory> ReactAgent<M> {
    pub fn new(config: &Config, model_name: &str, system_prompt: &str, max_iterations: usize, tool_manager: ToolManager, memory: M, tool_names: Vec<String>) -> Self {
        let model_config = config.models.get(model_name).expect(format!("Model {} not found in config", model_name).as_str());
        let model = LitellmModel::new(model_name, model_config, system_prompt);
        let tool_schemas = tool_manager.get_schema(&tool_names);

        let mut ret = Self {
            model,
            system_prompt: Message::system(system_prompt),
            max_iterations,
            tool_manager,
            tool_names,
            memory,
        };
        ret.system_prompt.content = ret.build_system_prompt(&ret.system_prompt.content);
        
        if tool_schemas.len() > 0 {
            let model = LitellmModel::new_with_tools(model_name, model_config, &ret.system_prompt.content, tool_schemas);
            ret.model = model;
        }else{
            let model = LitellmModel::new(model_name, model_config, &ret.system_prompt.content);
            ret.model = model;
        }
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
        self.get_history()
   }

   fn clear_history(&mut self) {
        self.memory.clear();
   }

   fn get_history(&self) -> impl Iterator<Item = &Message> {
        self.memory.get_messages()
   }

   async fn run(&mut self, user_prompt: &str) -> String{
        tracing::debug!("Current memory: {:?}", self.memory.get_messages().collect::<Vec<&Message>>());
        self.add_message(Message::user(user_prompt)).await;
        tracing::debug!("Running ReactAgent with user prompt: {}", user_prompt);

        for i in 0..self.max_iterations {
            let msgs: Vec<&Message> = self.build_messages().collect();
            tracing::debug!("Current memory: {:?}", self.memory.get_messages().collect::<Vec<&Message>>());
            tracing::debug!("Iteration {}/{}", i + 1, self.max_iterations);
            let response = self.model.call_with_history(msgs).await;
            
            // response is formmatted well for react agent
            if self.is_finished(&response){
                let final_answer = self.extract_final_answer(&response);
                if let Some(answer) = final_answer {
                    self.add_message(Message::assistant(&answer, None)).await;
                    tracing::debug!("Final answer extracted: {}", answer);
                    return answer;
                }
            }
            let content = response.content.as_ref().unwrap_or(&"Nothing".to_string()).to_string();
            let reasoning = response.reasoning_content.as_ref().unwrap_or(&"Nothing".to_string()).to_string();
            let content = format!("Content: {} Reasoning:{}", reasoning, content);
            // no tool calling
            if (&response).tool_calls.is_none(){
                self.add_message(Message::assistant(content.as_str(), None)).await;
                tracing::debug!("Response (no tools): {}", content.as_str());
                continue;
            }
            
            let tool_calls = response.tool_calls.unwrap();
            let formatted = self.tool_manager.format_tool_calls(
                tool_calls.iter().collect()
            );
            self.add_message(Message::assistant(&content, Some(tool_calls.clone()))).await;            
            
            tracing::debug!("Tool calls: {:?}", formatted);
            for tc in tool_calls.iter(){
                let id = &tc.id;
                let function_name = &tc.function.name;
                let arguments = &tc.function.arguments;
                tracing::debug!("Executing tool: {}#{}", id, function_name);
                let result = self.execute_tool(function_name, arguments).unwrap_or(String::from("No output from tool."));
                tracing::debug!("Tool result: {}#{:?}", id, result);
                self.add_message(Message::tool(&result, Some(vec![tc.clone()]), Some(id.clone()))).await;
            }

        }
        return String::from("Reached maximum iterations without a final answer.");
   }  
}


impl <M: BaseMemory + Send> ToolAgent for ReactAgent<M> {
    fn get_tools_schema(&self) -> Vec<Value> {
        self.tool_manager.get_schema(&self.tool_names)
    }
    fn format_tool_calls(&self, tool_calls: Vec<&ToolCall>) -> Vec<String> {
        self.tool_manager.format_tool_calls(tool_calls)
    }

    fn execute_tool(&self, tool_name: &str, arguments: &str) -> Option<String> {
        let tool = self.tool_manager.get_tool(tool_name)
                                           .with_context(|| format!("Tool {} not found", tool_name))
                                           .ok()?;
        let output = tool.execute(arguments);
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_react_agent_run() {
        let config = crate::config::config::load_config(None);
        let model_name = "gpt-4o-mini";
        let memory = crate::memory::summary::SummaryMemory::new("test-1", 0.3, &config, model_name, "", 8192, "./workspace_test/");
        let tool_manager = ToolManager::new(Vec::new());
        let mut agent = ReactAgent::new(
            &config,
            model_name,
            "You are a React Agent. Use tools to answer user queries.",
            3,
            tool_manager,
            memory,
            Vec::new()
        );

        let user_prompt = "Can you summarize the plot of 'Inception' and suggest a related movie?";
        let answer = agent.run(user_prompt).await;
        println!("Agent Answer: {}", answer);
    }

    #[tokio::test]
    async fn test_two_react_agents_run() {
        let config = crate::config::config::load_config(None);
        let model_name = "gpt-4o-mini";

        let memory = crate::memory::summary::SummaryMemory::new("test-1", 0.3, &config, model_name, "", 8192, "./workspace_test/");
        let tool_manager = ToolManager::new(Vec::new());
        let mut agent1 = ReactAgent::new(
            &config,
            model_name,
            "You are a React Agent.",
            3,
            tool_manager,
            memory,
            Vec::new()
        );

        let memory = crate::memory::summary::SummaryMemory::new("test-2", 0.3, &config, model_name, "", 8192, "./workspace_test/");
        let tool_manager = ToolManager::new(Vec::new());
        let mut agent2 = ReactAgent::new(
            &config,
            model_name,
            "You are another React Agent.",
            3,
            tool_manager,
            memory,
            Vec::new()
        );

        let user_prompt = "Can you summarize some movies about dreams?";
        let answer = agent1.run(user_prompt).await;
        println!("Agent Answer: {}", answer);
        let follow_up_prompt = format!("Based on the previous answer: {}, can you suggest a movie?", answer);
        let follow_up_answer = agent2.run(&follow_up_prompt).await;
        println!("Follow-up Agent Answer: {}", follow_up_answer);
    }
}