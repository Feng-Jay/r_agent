use async_trait::async_trait;

#[async_trait]
pub trait BaseModel {
    async fn call(&self, user_prompt: &str, system_prompt: Option<&str>) -> String;
    async fn call_with_history(
        &self,
        user_prompt: &str,
        history: Vec<&str>,
        system_prompt: Option<&str>,
    ) -> String;
}