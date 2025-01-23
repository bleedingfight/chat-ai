use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use log::{debug, error};
use std::time::Instant;
use futures_util::StreamExt;
use std::fs;
use crate::chat::{ChatMessage, ChatPayload, StreamResponse};
use crate::models::{AvailableModelsResponse, ModelsResponse, ModelFrequency};
use crate::cache::{get_cache_dir, MODEL_FREQUENCIES, update_frequency};

#[tauri::command]
pub async fn chat(_app_handle: tauri::AppHandle, message: String, api_key: String, api_url: String, model: String, history: Vec<ChatMessage>) -> Result<String, String> {
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
        update_frequency(model, false);
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

    update_frequency(model, true);

    let elapsed = start_time.elapsed();
    Ok(format!("{}\n<p style=\"color: green\">响应耗时: {:.2}秒</p>", 
        response_text, 
        elapsed.as_secs_f64()
    ))
}

#[tauri::command]
pub async fn fetch_models(api_url: String, api_key: String) -> Result<AvailableModelsResponse, String> {
    let cache_dir = get_cache_dir();
    let frequency_file = cache_dir.join("frequency.json");
    
    // 尝试从缓存文件读取模型频率数据
    if frequency_file.exists() {
        if let Ok(content) = fs::read_to_string(&frequency_file) {
            if let Ok(frequency_data) = serde_json::from_str::<ModelFrequency>(&content) {
                // 过滤掉value为-1的模型，并按照使用频率排序
                let mut model_frequencies: Vec<(String, i32)> = frequency_data.frequencies
                    .into_iter()
                    .filter(|(_, freq)| *freq != -1)
                    .collect();
                
                // 按频率降序排序
                model_frequencies.sort_by(|a, b| b.1.cmp(&a.1));
                
                let models = model_frequencies
                    .into_iter()
                    .map(|(model, _)| model)
                    .collect();
                
                return Ok(AvailableModelsResponse { models });
            }
        }
    }
    
    // 如果缓存文件不存在或读取失败，则从API获取模型列表
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

    let mut models: Vec<String> = models_response.data
        .into_iter()
        .map(|model| model.id)
        .collect();
    
    // 对模型列表进行字母顺序排序
    models.sort();

    // 初始化模型频率
    let mut frequencies = MODEL_FREQUENCIES.lock().unwrap();
    for model in &models {
        frequencies.entry(model.clone()).or_insert(0);
    }

    Ok(AvailableModelsResponse { models })
}