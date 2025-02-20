use chat_ai_lib::handlers::{
    save_api_key,
    get_api_key,
    remove_api_key,
    save_api_url,
    get_api_url,
    remove_api_url,
};
use chat_ai_lib::chat::{ChatMessage, ChatPayload};
use chat_ai_lib::models::{ModelsResponse, ModelData, AvailableModelsResponse};

#[test]
fn test_api_key_management() {
    let test_key = "test-api-key-123".to_string();
    
    // 测试保存 API key
    assert!(save_api_key(test_key.clone()).is_ok());
    
    // 测试获取 API key
    let retrieved_key = get_api_key();
    assert!(retrieved_key.is_ok());
    assert_eq!(retrieved_key.unwrap(), test_key);
    
    // 测试删除 API key
    assert!(remove_api_key().is_ok());
    
    // 验证删除后无法获取
    let result = get_api_key();
    assert!(result.is_err());
}

#[test]
fn test_api_url_management() {
    let test_url = "https://test.api.com".to_string();
    
    // 测试保存 API URL
    assert!(save_api_url(test_url.clone()).is_ok());
    
    // 测试获取 API URL
    let retrieved_url = get_api_url();
    assert!(retrieved_url.is_ok());
    assert_eq!(retrieved_url.unwrap(), test_url);
    
    // 测试删除 API URL
    assert!(remove_api_url().is_ok());
    
    // 验证删除后无法获取
    let result = get_api_url();
    assert!(result.is_err());
}

#[test]
fn test_chat_payload() {
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }
    ];
    
    let payload = ChatPayload {
        model: "gpt-3.5-turbo".to_string(),
        messages: messages.clone(),
        stream: true,
    };
    
    assert_eq!(payload.model, "gpt-3.5-turbo");
    assert_eq!(payload.messages.len(), 1);
    assert_eq!(payload.messages[0].role, "user");
    assert_eq!(payload.messages[0].content, "Hello");
    assert!(payload.stream);
}

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
fn test_invalid_api_key() {
    // 设置无效的 API key
    let _ = save_api_key("invalid-key".to_string());
    
    // 验证可以获取到无效的 key
    let result = get_api_key();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "invalid-key");
    
    // 清理
    let _ = remove_api_key();
}

#[test]
fn test_empty_api_key() {
    // 测试空 API key
    let result = save_api_key("".to_string());
    assert!(result.is_err());
}

#[test]
fn test_empty_api_url() {
    // 测试空 API URL
    let result = save_api_url("".to_string());
    assert!(result.is_err());
}