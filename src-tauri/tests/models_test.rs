use chat_ai_lib::models::{ModelsResponse, ModelData, AvailableModelsResponse, ModelFrequency};
use std::collections::HashMap;

#[test]
fn test_models_response() {
    let model_data = ModelData {
        id: "gpt-3.5-turbo".to_string(),
    };
    
    let models_response = ModelsResponse {
        data: vec![model_data],
    };
    
    assert_eq!(models_response.data.len(), 1);
    assert_eq!(models_response.data[0].id, "gpt-3.5-turbo");
}

#[test]
fn test_available_models_response() {
    let models = vec!["gpt-3.5-turbo".to_string(), "gpt-4".to_string()];
    let response = AvailableModelsResponse {
        models: models.clone(),
    };
    
    assert_eq!(response.models.len(), 2);
    assert_eq!(response.models, models);
}

#[test]
fn test_model_frequency() {
    let mut frequencies = HashMap::new();
    frequencies.insert("gpt-3.5-turbo".to_string(), 5);
    frequencies.insert("gpt-4".to_string(), 3);
    
    let model_frequency = ModelFrequency {
        frequencies,
    };
    
    assert_eq!(model_frequency.frequencies.len(), 2);
    assert_eq!(model_frequency.frequencies.get("gpt-3.5-turbo"), Some(&5));
    assert_eq!(model_frequency.frequencies.get("gpt-4"), Some(&3));
}

#[test]
fn test_model_frequency_serialization() {
    let mut frequencies = HashMap::new();
    frequencies.insert("gpt-3.5-turbo".to_string(), 5);
    frequencies.insert("gpt-4".to_string(), 3);
    
    let model_frequency = ModelFrequency {
        frequencies,
    };
    
    let serialized = serde_json::to_string(&model_frequency).unwrap();
    let deserialized: ModelFrequency = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(model_frequency.frequencies, deserialized.frequencies);
}

#[test]
fn test_models_response_serialization() {
    let model_data = ModelData {
        id: "gpt-3.5-turbo".to_string(),
    };
    
    let models_response = ModelsResponse {
        data: vec![model_data],
    };
    
    let serialized = serde_json::to_string(&models_response).unwrap();
    let deserialized: ModelsResponse = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(models_response.data.len(), deserialized.data.len());
    assert_eq!(models_response.data[0].id, deserialized.data[0].id);
}

#[test]
fn test_available_models_response_serialization() {
    let models = vec!["gpt-3.5-turbo".to_string(), "gpt-4".to_string()];
    let response = AvailableModelsResponse {
        models: models.clone(),
    };
    
    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: AvailableModelsResponse = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(response.models, deserialized.models);
}