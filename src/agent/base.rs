use crate::model::litellm_model::Litellm_Model;
use crate::model::base::BaseModel;
use crate::model::schema::*;


pub struct BaseAgent {
    pub model: Litellm_Model,
    pub system_prompt: String,
}

impl BaseAgent {
    fn new(model: Litellm_Model, system_prompt: String) -> Self {
        BaseAgent {
            model,
            system_prompt
        }
    }
}