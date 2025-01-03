// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[tauri::command]
async fn chat(message: String, api_key: String, api_url: String, model: String) -> Result<String, String> {
    if message.trim().is_empty() {
        return Err("Message cannot be empty".to_string());
    }

    if api_key.trim().is_empty() {
        return Err("API Key cannot be empty".to_string());
    }

    if api_url.trim().is_empty() {
        return Err("API URL cannot be empty".to_string());
    }

    let client = reqwest::Client::new();
    
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": model,
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": message}
            ]
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API request failed ({}): {}", status, error_text));
    }

    let chat_response = response
        .json::<ChatResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    chat_response.choices.first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| "No response from AI".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
