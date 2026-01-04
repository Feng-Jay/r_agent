use serde::Serialize;
use std::{collections::HashMap, fmt::Debug};

#[derive(Serialize, Debug, Clone)]
pub struct ToolParameters {
    // rename
    #[serde(rename = "type")]
    pub type_: String,
    pub properties: HashMap<String, ToolParametersPropoerty>,
    pub reqiured: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ToolParametersPropoerty {
    pub type_: String,
    pub description: String,
}

pub trait Tool: Debug + Send{
    fn name(&self) -> &str;
    fn type_(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> &ToolParameters;
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