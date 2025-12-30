use std::fmt;

#[derive(Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
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
        }
    }

    pub fn user(content: &str) -> Self {
        Message {
            role: Role::USER,
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Message {
            role: Role::ASSISTANT,
            content: content.to_string(),
        }
    }

    pub fn tool(content: &str) -> Self {
        Message {
            role: Role::TOOL,
            content: content.to_string(),
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
}

#[derive(Debug)]
pub struct Usage{
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}