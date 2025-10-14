use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<ModelData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelData {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableModelsResponse {
    pub models: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelFrequency {
    pub frequencies: HashMap<String, i32>,
}