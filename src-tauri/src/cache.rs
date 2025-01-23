use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::env;
use std::sync::Mutex;
use log::error;
use lazy_static::lazy_static;
use crate::models::ModelFrequency;

lazy_static! {
    pub static ref MODEL_FREQUENCIES: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
}

pub fn get_cache_dir() -> PathBuf {
    if let Ok(cache_dir) = env::var("CHATAICACHE") {
        PathBuf::from(cache_dir)
    } else {
        let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
        PathBuf::from(home).join(".cache/chat-ai")
    }
}

pub fn save_frequencies() {
    let frequencies = MODEL_FREQUENCIES.lock().unwrap();
    let frequency_data = ModelFrequency {
        frequencies: frequencies.clone(),
    };
    
    let cache_dir = get_cache_dir();
    
    // 确保缓存目录存在
    if let Err(e) = fs::create_dir_all(&cache_dir) {
        error!("创建缓存目录失败: {}", e);
        return;
    }
    
    let frequency_file = cache_dir.join("frequency.json");
    
    if let Ok(json) = serde_json::to_string_pretty(&frequency_data) {
        if let Err(e) = fs::write(frequency_file, json) {
            error!("保存频率数据失败: {}", e);
        }
    }
}

pub fn update_frequency(model: String, success: bool) {
    let mut frequencies = MODEL_FREQUENCIES.lock().unwrap();
    if success {
        let count = frequencies.entry(model).or_insert(0);
        *count += 1;
    } else {
        frequencies.insert(model, -1);
    }
}