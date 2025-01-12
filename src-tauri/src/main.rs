// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use log::{info, error};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    message: String,
    api_key: String,
    api_url: String,
    model: String,
    history: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatPayload {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Choice {
    message: ChatMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelData {
    id: String,
}

#[derive(Debug, Serialize)]
struct AvailableModelsResponse {
    models: Vec<String>,
}

#[tauri::command]
async fn chat(message: String, api_key: String, api_url: String, model: String, history: Vec<ChatMessage>) -> Result<String, String> {
    info!("收到请求:");
    info!("API URL: {}", api_url);
    info!("Model: {}", model);
    info!("Message: {}", message);
    info!("API Key: {}****", &api_key[..4]);
    
    let client = reqwest::Client::new();
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| e.to_string())?
    );

    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: message.clone(),
        }
    ];

    let payload = ChatPayload {
        model: model.clone(),
        messages: messages.clone(),
    };

    info!("发送到 API 的数据: {:?}", payload);

    let response = client
        .post(&api_url)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let error_msg = format!("API request failed with status: {}", response.status());
        error!("请求失败: {}", error_msg);
        return Err(error_msg);
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    info!("API 响应: {:?}", chat_response);

    Ok(chat_response.choices[0].message.content.clone())
}

#[tauri::command]
async fn fetch_models(api_url: String, api_key: String) -> Result<AvailableModelsResponse, String> {
    let client = reqwest::Client::new();
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| e.to_string())?
    );

    let base_url = if api_url.ends_with("/") {
        api_url.trim_end_matches("/").to_string()
    } else {
        api_url.to_string()
    };
    
    let models_url = if base_url.contains("/v1/chat/completions") {
        base_url.replace("/chat/completions", "/models")
    } else {
        if base_url.ends_with("/v1") {
            format!("{}/models", base_url)
        } else {
            format!("{}/v1/models", base_url.trim_end_matches("/v1"))
        }
    };

    let response = client
        .get(&models_url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()));
    }

    let models_response: ModelsResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let models = models_response.data
        .into_iter()
        .map(|model| model.id)
        .collect();

    Ok(AvailableModelsResponse { models })
}

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![chat, fetch_models])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
