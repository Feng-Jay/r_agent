#[derive(Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
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