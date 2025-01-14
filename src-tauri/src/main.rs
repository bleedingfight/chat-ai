// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use log::{debug, error, info};
use std::env;
use std::time::Instant;
use futures_util::StreamExt;
use tauri::Manager;

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
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StreamChoice {
    delta: DeltaContent,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeltaContent {
    content: Option<String>,
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
async fn chat(app_handle: tauri::AppHandle, message: String, api_key: String, api_url: String, model: String, history: Vec<ChatMessage>) -> Result<String, String> {
    debug!("收到请求:");
    debug!("API URL: {}", api_url);
    debug!("Model: {}", model);
    debug!("Message: {}", message);
    debug!("API Key: {}****", &api_key[..4]);
    
    let start_time = Instant::now();
    
    let client = reqwest::Client::new();
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| e.to_string())?
    );

    let mut messages = history;
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: message.clone(),
    });

    let payload = ChatPayload {
        model: model.clone(),
        messages,
        stream: true,
    };

    debug!("发送到 API 的数据: {:?}", payload);

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

    let mut response_text = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        let chunk_str = String::from_utf8_lossy(&chunk);
        
        for line in chunk_str.lines() {
            if line.starts_with("data: ") {
                let data = &line["data: ".len()..];
                if data == "[DONE]" {
                    continue;
                }
                
                if let Ok(stream_response) = serde_json::from_str::<StreamResponse>(data) {
                    if let Some(choice) = stream_response.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            response_text.push_str(content);
                        }
                    }
                }
            }
        }
    }

    let elapsed = start_time.elapsed();
    Ok(format!("{}\n<p style=\"color: green\">响应耗时: {:.2}秒</p>", 
        response_text, 
        elapsed.as_secs_f64()
    ))
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
    #[cfg(debug_assertions)]
    {
        env::set_var("RUST_LOG", "info");
        env_logger::init();
    }

    let app = tauri::Builder::default()
        .setup(|_| Ok(()))
        .invoke_handler(tauri::generate_handler![chat, fetch_models])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::Ready => {}
        _ => {}
    });
}
