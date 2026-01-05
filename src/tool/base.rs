use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolParameters {
    // rename
    #[serde(rename = "type")]
    pub type_: String,
    pub properties: HashMap<String, ToolParametersPropoerty>,
    pub reqiured: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolParametersPropoerty {
    pub type_: String,
    pub description: String,
}

pub trait Tool: Debug + Send{
    fn load(&self) -> &Value;
    fn name(&self) -> &str {self.load().get("name").and_then(Value::as_str).unwrap_or("empty_name")}
    fn type_(&self) -> &str {self.load().get("type").and_then(Value::as_str).unwrap_or("empty_type")}
    fn description(&self) -> &str {self.load().get("description").and_then(Value::as_str).unwrap_or("empty_description")}
    fn parameters(&self) -> &Value{ self.load().get("parameters").unwrap_or(&Value::Null) }
    fn init(&mut self);
    fn execute(&self, input: &str) -> String;
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tool_parameters() {
        let mut properties = HashMap::new();
        properties.insert(
            "location".to_string(),
            ToolParametersPropoerty {
                type_: "string".to_string(),
                description: "The location to get the weather for".to_string(),
            },
        );
        let parameters = ToolParameters {
            type_: "object".to_string(),
            properties,
            reqiured: vec!["location".to_string()],
        };
        assert_eq!(parameters.type_, "object");
        assert_eq!(parameters.reqiured.len(), 1);
        dbg!(parameters);
    }
}