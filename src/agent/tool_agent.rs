use llm::ToolCall;

use serde_json::Value;
use async_trait::async_trait;

pub trait ToolAgent {
    fn get_tools_schema(&self) -> Vec<Value>;
    fn format_tool_calls(&self, tool_calls: Vec<&ToolCall>) -> Vec<String>;
    fn execute_tool(&self, tool_name: &str, arguments: &str) -> Option<String>;
}