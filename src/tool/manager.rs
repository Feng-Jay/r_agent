use std::{collections::HashMap, sync::{Arc}, vec::Vec};
use crate::tool::base::{Tool, ToolParameters};
use async_trait::async_trait;
use tokio::sync::Mutex;
use llm::ToolCall;
use serde_json::{json, Value};

#[derive(Debug)]
pub struct ToolManager {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolManager {
    pub fn new(mut tools: Vec<Box<dyn Tool>>) -> Self {
        ToolManager {
            tools: tools.drain(..).fold(
                HashMap::new(),
                |mut acc, tool| {
                    let tool_name = tool.name().to_string();
                    acc.insert(tool_name, tool);
                    acc
                },
            ),
        }
    }

    pub fn get_tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub fn get_tool(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.get(name)
    }

    pub fn get_schema(&self, names: &Vec<String>) -> Vec<Value> {
        // provide tool schema for LLM
        let mut schemas:Vec<Value> = Vec::new();
        for name in names {
            if let Some(tool) = self.tools.get(name) {
                // let tool = tool
                schemas.push(Self::tool_to_schema(&tool));
            }
        }
        schemas
    }

    pub fn format_tool_calls(&self, tool_calls: Vec<&ToolCall>) -> Vec<String> {
        // provide strings for message backup
        let mut ret = Vec::new();
        for tool_call in tool_calls {
            ret.push(serde_json::to_string(tool_call).unwrap());
        }
        ret
    }

    pub fn clear(&mut self) {
        self.tools.clear();
    }

    pub fn tool_to_schema(tool: &Box<dyn Tool>) -> Value{
        let schema = json!(
            {
                "name": tool.name(),
                "description": tool.description(),
                "parameters": tool.parameters()
            }
        );
        schema
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyTool;
    
    impl Tool for DummyTool {
        fn name(&self) -> &str {
            "dummy_tool"
        }
        fn type_(&self) -> &str {
            "object"
        }
        fn description(&self) -> &str {
            "A dummy tool for testing"
        }
        fn parameters(&self) -> &crate::tool::base::ToolParameters {
            unimplemented!()
        }
        fn init(&mut self) {}

        fn execute(&self, _input: &str) -> String {
            "dummy_output".to_string()
        }
    }

    #[test]
    fn test_tool_manager() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(DummyTool)];
        let manager = ToolManager::new(tools);
        dbg!(manager);
    }
}
