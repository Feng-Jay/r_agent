use r_agent::agent::base::BaseAgent;
use r_agent::tool::base::Tool;
use serde_json::Value;
use serde_json::json;
use r_agent::config::config::*;
use serde_json;

#[derive(Debug)]
struct CalculatorTool{
    config: Value
}

impl Tool for CalculatorTool {
    fn load(&self) -> &serde_json::Value {
        &self.config
    }

    fn init(&mut self) {
        // Initialization logic if needed
    }

    fn execute(&self, input: &str) -> String {
        // A very basic implementation that only handles addition for demonstration
        let args = serde_json::from_str::<serde_json::Value>(input).unwrap();
        String::from(
            format!("{}", args["num1"].as_f64().unwrap() + args["num2"].as_f64().unwrap())
        )
    }
}

#[tokio::main]
async fn main() {
    let tool = CalculatorTool {
        config: json!({
            "name": "sumOfTwoNumbers",
            "type": "object",
            "description": "A tool to calculate the sum of two numbers.",
            "parameters": {
                "type": "object",
                "properties": {
                    "num1": {
                        "type": "number",
                        "description": "The first number."
                    },
                    "num2": {
                        "type": "number",
                        "description": "The second number."
                    }
                },
                "required": ["num1", "num2"]
            }
        })
    };

    let config = load_config(None);
    let model_name = "gpt-4o-mini";
    
    let memory = r_agent::memory::summary::SummaryMemory::new("tool_agent_example", 0.3, &config, model_name, "", 8192, "./workspace/");
    
    let system_prompt = "You are a React Agent. Use tools to answer user queries.";
    let tool_manager = r_agent::tool::manager::ToolManager::new(vec![Box::new(tool)]);
    let tools = vec!["sumOfTwoNumbers".to_string()];

    let mut agent = r_agent::agent::react_agent::ReactAgent::new(
        &config,
        model_name,
        system_prompt,
        3,
        tool_manager,
        memory,
        tools,
    );

    let user_prompt = "What is the sum of 1000 and 10000? Use the tool and finally give me the answer.";

    let answer = agent.run(user_prompt).await;
    println!("Agent Answer: {}", answer);
}
