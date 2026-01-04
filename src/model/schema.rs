use std::fmt;
use llm::ToolCall;

#[derive(Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.role, self.content)
    }
}

impl Message {
    pub fn system(content: &str) -> Self {
        Message {
            role: Role::SYSTEM,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: &str) -> Self {
        Message {
            role: Role::USER,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: &str, tool_calls:Option<Vec<ToolCall>>) -> Self {
        Message {
            role: Role::ASSISTANT,
            content: content.to_string(),
            tool_calls,
            tool_call_id: None,
        }
    }

    pub fn tool(content: &str, tool_calls:Option<Vec<ToolCall>>, tool_call_id:Option<String>) -> Self {
        Message {
            role: Role::TOOL,
            content: content.to_string(),
            tool_calls,
            tool_call_id
        }
    }
}

#[derive(Debug)]
pub enum Role{
    SYSTEM,
    USER,
    ASSISTANT,
    TOOL
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role_str = match self {
            Role::SYSTEM => "system",
            Role::USER => "user",
            Role::ASSISTANT => "assistant",
            Role::TOOL => "tool",
        };
        write!(f, "{}", role_str)
    }
}

#[derive(Debug)]
pub struct LLMResponse {
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub usage: Option<Usage>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug)]
pub struct Usage{
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}


