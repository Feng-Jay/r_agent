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
        let tool_calls_str = match &self.tool_calls {
            Some(calls) => format!("{:?}", calls),
            None => "None".to_string(),
        };
        let tool_call_id_str = match &self.tool_call_id {
            Some(id) => id.clone(),
            None => "None".to_string(),
        };
        write!(f, "{}: content: {}, tool_calls: {}, tool_call_id: {}\n", self.role, self.content, tool_calls_str, tool_call_id_str)
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

impl fmt::Display for LLMResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut write_str = String::from("LLM Response:\n");
        if let Some(content) = &self.content {
            write_str.push_str(&format!("Content: {}\n", content));
        } else {
            write_str.push_str("Content: None\n");
        }
        if let Some(reasoning_content) = &self.reasoning_content {
            write_str.push_str(&format!("Reasoning Content: {}\n", reasoning_content));
        } else {
            write_str.push_str("Reasoning Content: None\n");
        }
        if let Some(usage) = &self.usage {
            write_str.push_str(&format!("Usage: {}\n", usage));
        } else {
            write_str.push_str("Usage: None\n");
        }
        if let Some(tool_calls) = &self.tool_calls {
            write_str.push_str(&format!("Tool Calls: {:#?}\n", tool_calls));
        } else {
            write_str.push_str("Tool Calls: None\n");
        }
        write!(f, "{}", write_str)
            
    }
}

#[derive(Debug)]
pub struct Usage{
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cost_usd: f64,
}

impl fmt::Display for Usage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prompt_tokens: {}, completion_tokens: {}, total_tokens: {}, cost_usd: {}", self.prompt_tokens, self.completion_tokens, self.total_tokens, self.cost_usd)
    }
}


