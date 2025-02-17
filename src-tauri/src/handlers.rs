use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use log::{debug, error};
use std::time::Instant;
use futures_util::StreamExt;
use std::fs;
use tauri::{Window, Emitter};
use crate::chat::{ChatMessage, ChatPayload, StreamResponse};
use crate::models::{AvailableModelsResponse, ModelsResponse, ModelFrequency};
use crate::cache::{get_cache_dir, MODEL_FREQUENCIES, update_frequency, encrypt_api_key, decrypt_api_key, delete_api_key, encrypt_api_url, decrypt_api_url, delete_api_url};

#[tauri::command]
pub async fn chat(window: Window, message: String, api_key: String, api_url: String, model: String, history: Vec<ChatMessage>) -> Result<String, String> {
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
        update_frequency(model.clone(), false);
        return Err(error_msg);
    }

    let mut stream = response.bytes_stream();
    let mut total_content = String::new();

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
                            // 发送流式内容到前端
                            window.emit("stream-response", &content).map_err(|e| e.to_string())?;
                            total_content.push_str(content);
                        }
                    }
                }
            }
        }
    }

    update_frequency(model, true);

    let elapsed = start_time.elapsed();
    Ok(format!("流式响应完成\n<p style=\"color: green\">响应耗时: {:.2}秒</p>", elapsed.as_secs_f64()))
}

/// 从 API 获取模型列表
async fn fetch_models_from_api(api_url: &str, api_key: &str) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| e.to_string())?
    );

    // 构建 models API URL
    let models_url = if api_url.ends_with("/v1/chat/completions") {
        api_url.replace("/chat/completions", "/models")
    } else if api_url.ends_with("/chat/completions") {
        api_url.replace("/chat/completions", "/models")
    } else if api_url.ends_with("/v1") {
        format!("{}/models", api_url)
    } else if api_url.ends_with("/v1/") {
        format!("{}models", api_url)
    } else {
        format!("{}/v1/models", api_url.trim_end_matches('/'))
    };

    debug!("Models API URL: {}", models_url);

    let response = client
        .get(&models_url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| {
            error!("获取模型列表失败: {}", e);
            e.to_string()
        })?;

    if !response.status().is_success() {
        let error_msg = format!("获取模型列表失败，状态码: {}，URL: {}", response.status(), models_url);
        error!("{}", error_msg);
        return Err(error_msg);
    }

    let response_text = response.text().await.map_err(|e| {
        error!("解析响应失败: {}", e);
        e.to_string()
    })?;

    debug!("API 响应: {}", response_text);

    let models_response: ModelsResponse = serde_json::from_str(&response_text).map_err(|e| {
        error!("解析 JSON 失败: {}", e);
        format!("解析模型列表失败: {}. 响应内容: {}", e, response_text)
    })?;

    let mut models: Vec<String> = models_response.data
        .into_iter()
        .map(|model| model.id)
        .collect();
    
    // 对模型列表进行字母顺序排序
    models.sort();

    Ok(models)
}

/// 将模型列表写入配置文件
fn write_models_to_file(models: &[String], frequency_file: &std::path::Path) -> Result<(), String> {
    let mut frequencies = MODEL_FREQUENCIES.lock().unwrap();
    
    // 初始化或更新模型频率
    for model in models {
        frequencies.entry(model.clone()).or_insert(0);
    }
    
    // 将频率数据写入文件
    let frequency_data = ModelFrequency {
        frequencies: frequencies.clone(),
    };
    
    serde_json::to_string_pretty(&frequency_data)
        .map_err(|e| format!("序列化频率数据失败: {}", e))
        .and_then(|json| {
            fs::write(frequency_file, json)
                .map_err(|e| format!("写入频率文件失败: {}", e))
        })
}

#[tauri::command]
pub async fn fetch_models(api_url: String, api_key: String) -> Result<AvailableModelsResponse, String> {
    let cache_dir = get_cache_dir();
    let frequency_file = cache_dir.join("frequency.json");
    
    // 尝试从配置文件读取模型列表
    let models = if frequency_file.exists() {
        // 读取并解析配置文件
        match fs::read_to_string(&frequency_file)
            .map_err(|e| format!("读取配置文件失败: {}", e))
            .and_then(|content| {
                serde_json::from_str::<ModelFrequency>(&content)
                    .map_err(|e| format!("解析配置文件失败: {}", e))
            }) {
            Ok(frequency_data) => {
                // 过滤掉 value=-1 的模型
                let valid_models: Vec<String> = frequency_data.frequencies
                    .into_iter()
                    .filter(|(_, freq)| *freq != -1)
                    .map(|(model, _)| model)
                    .collect();
                
                if valid_models.is_empty() {
                    // 如果过滤后没有有效模型，从 API 获取
                    debug!("配置文件中没有有效模型，从 API 获取");
                    let models = fetch_models_from_api(&api_url, &api_key).await?;
                    write_models_to_file(&models, &frequency_file)?;
                    models
                } else {
                    valid_models
                }
            }
            Err(e) => {
                // 如果解析失败，从 API 获取
                debug!("解析配置文件失败: {}，从 API 获取", e);
                let models = fetch_models_from_api(&api_url, &api_key).await?;
                write_models_to_file(&models, &frequency_file)?;
                models
            }
        }
    } else {
        // 如果配置文件不存在，从 API 获取并写入
        debug!("配置文件不存在，从 API 获取");
        let models = fetch_models_from_api(&api_url, &api_key).await?;
        write_models_to_file(&models, &frequency_file)?;
        models
    };

    Ok(AvailableModelsResponse { models })
}

#[tauri::command]
pub fn save_api_key(api_key: String) -> Result<(), String> {
    encrypt_api_key(&api_key)
}

#[tauri::command]
pub fn get_api_key() -> Result<String, String> {
    decrypt_api_key()
}

#[tauri::command]
pub fn remove_api_key() -> Result<(), String> {
    delete_api_key()
}

#[tauri::command]
pub fn save_api_url(api_url: String) -> Result<(), String> {
    encrypt_api_url(&api_url)
}

#[tauri::command]
pub fn get_api_url() -> Result<String, String> {
    decrypt_api_url()
}

#[tauri::command]
pub fn remove_api_url() -> Result<(), String> {
    delete_api_url()
}