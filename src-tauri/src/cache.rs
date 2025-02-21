use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::env;
use std::sync::Mutex;
use log::error;
use lazy_static::lazy_static;
use crate::models::ModelFrequency;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;

const KEY_FILE: &str = "encryption.key";
const API_KEYS_FILE: &str = "api_keys.enc";
const API_URL_FILE: &str = "api_url.enc";

lazy_static! {
    static ref CIPHER: Mutex<Option<Aes256Gcm>> = Mutex::new(None);
}

fn init_cipher() -> Result<(), String> {
    let mut cipher = CIPHER.lock().map_err(|e| e.to_string())?;
    if cipher.is_some() {
        return Ok(());
    }

    let cache_dir = get_cache_dir();
    let key_path = cache_dir.join(KEY_FILE);

    let key = if key_path.exists() {
        // 读取现有密钥
        fs::read(&key_path).map_err(|e| format!("无法读取加密密钥: {}", e))?
    } else {
        // 生成新密钥
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        fs::create_dir_all(&cache_dir).map_err(|e| format!("无法创建缓存目录: {}", e))?;
        fs::write(&key_path, &key).map_err(|e| format!("无法保存加密密钥: {}", e))?;
        key.to_vec()
    };

    *cipher = Some(Aes256Gcm::new_from_slice(&key).map_err(|e| format!("初始化加密失败: {}", e))?);
    Ok(())
}

lazy_static! {
    pub static ref MODEL_FREQUENCIES: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
}

pub fn get_cache_dir() -> PathBuf {
    if let Ok(cache_dir) = env::var("CHATAICACHE") {
        PathBuf::from(cache_dir)
    } else {
        if cfg!(target_os = "windows") {
            // Windows: 使用 %LOCALAPPDATA%\chat-ai
            if let Ok(app_data) = env::var("LOCALAPPDATA") {
                PathBuf::from(app_data).join("chat-ai")
            } else {
                // 降级方案：使用 %USERPROFILE%\AppData\Local\chat-ai
                let user_profile = env::var("USERPROFILE")
                    .unwrap_or_else(|_| String::from("C:\\"));
                PathBuf::from(user_profile)
                    .join("AppData")
                    .join("Local")
                    .join("chat-ai")
            }
        } else {
            // Unix/Linux/macOS: 使用 ~/.cache/chat-ai
            let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
            PathBuf::from(home).join(".cache/chat-ai")
        }
    }
}

pub fn save_frequencies(frequency_file: PathBuf) {
    let frequencies = MODEL_FREQUENCIES.lock().unwrap();
    let frequency_data = ModelFrequency {
        frequencies: frequencies.clone(),
    };
    
    // 确保父目录存在
    if let Some(parent) = frequency_file.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            error!("创建目录失败: {}", e);
            return;
        }
    }
    
    if let Ok(json) = serde_json::to_string_pretty(&frequency_data) {
        if let Err(e) = fs::write(&frequency_file, json) {
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

pub fn encrypt_api_key(api_key: &str) -> Result<(), String> {
    init_cipher()?;
    
    let cipher = CIPHER.lock().map_err(|e| e.to_string())?;
    let cipher = cipher.as_ref().unwrap();
    
    // 生成随机 nonce
    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);
    
    // 加密数据
    let encrypted = cipher
        .encrypt(nonce, api_key.as_bytes())
        .map_err(|e| format!("加密失败: {}", e))?;
    
    // 将 nonce 和加密数据合并并进行 base64 编码
    let mut combined = nonce.to_vec();
    combined.extend(encrypted);
    let encoded = BASE64.encode(combined);
    
    // 保存加密数据
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir).map_err(|e| format!("无法创建缓存目录: {}", e))?;
    fs::write(cache_dir.join(API_KEYS_FILE), encoded).map_err(|e| format!("无法保存加密数据: {}", e))?;
    
    Ok(())
}

pub fn decrypt_api_key(api_keys_path: PathBuf) -> Result<String, String> {
    init_cipher()?;
    
    let cipher = CIPHER.lock().map_err(|e| e.to_string())?;
    let cipher = cipher.as_ref().unwrap();
    
    if !api_keys_path.exists() {
        return Err("API key 未设置".to_string());
    }
    
    let encrypted = fs::read(&api_keys_path)
        .map_err(|e| format!("无法读取加密数据: {}", e))?;
    
    // base64 解码
    let decoded = BASE64.decode(encrypted)
        .map_err(|e| format!("无法解码数据: {}", e))?;
    
    if decoded.len() < 12 {
        return Err("无效的加密数据".to_string());
    }
    
    // 分离 nonce 和加密数据
    let (nonce, encrypted_data) = decoded.split_at(12);
    let nonce = Nonce::from_slice(nonce);
    
    // 解密数据
    let decrypted = cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("解密失败: {}", e))?;
    
    String::from_utf8(decrypted)
        .map_err(|e| format!("无法解析解密数据: {}", e))
}

pub fn encrypt_api_url(api_url: &str) -> Result<(), String> {
    init_cipher()?;
    
    let cipher = CIPHER.lock().map_err(|e| e.to_string())?;
    let cipher = cipher.as_ref().unwrap();
    
    // 生成随机 nonce
    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);
    
    // 加密数据
    let encrypted = cipher
        .encrypt(nonce, api_url.as_bytes())
        .map_err(|e| format!("加密失败: {}", e))?;
    
    // 将 nonce 和加密数据合并并进行 base64 编码
    let mut combined = nonce.to_vec();
    combined.extend(encrypted);
    let encoded = BASE64.encode(combined);
    
    // 保存加密数据
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir).map_err(|e| format!("无法创建缓存目录: {}", e))?;
    fs::write(cache_dir.join(API_URL_FILE), encoded).map_err(|e| format!("无法保存加密数据: {}", e))?;
    
    Ok(())
}

pub fn decrypt_api_url() -> Result<String, String> {
    init_cipher()?;
    
    let cipher = CIPHER.lock().map_err(|e| e.to_string())?;
    let cipher = cipher.as_ref().unwrap();
    
    // 读取加密数据
    let cache_dir = get_cache_dir();
    let api_url_path = cache_dir.join(API_URL_FILE);
    
    if !api_url_path.exists() {
        return Err("API URL 未设置".to_string());
    }
    
    let encrypted = fs::read(api_url_path)
        .map_err(|e| format!("无法读取加密数据: {}", e))?;
    
    // base64 解码
    let decoded = BASE64.decode(encrypted)
        .map_err(|e| format!("无法解码数据: {}", e))?;
    
    if decoded.len() < 12 {
        return Err("无效的加密数据".to_string());
    }
    
    // 分离 nonce 和加密数据
    let (nonce, encrypted_data) = decoded.split_at(12);
    let nonce = Nonce::from_slice(nonce);
    
    // 解密数据
    let decrypted = cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("解密失败: {}", e))?;
    
    String::from_utf8(decrypted)
        .map_err(|e| format!("无法解析解密数据: {}", e))
}

pub fn delete_api_key() -> Result<(), String> {
    let cache_dir = get_cache_dir();
    let api_keys_path = cache_dir.join(API_KEYS_FILE);
    
    if api_keys_path.exists() {
        fs::remove_file(api_keys_path)
            .map_err(|e| format!("无法删除 API key: {}", e))?;
    }
    
    Ok(())
}

pub fn delete_api_url() -> Result<(), String> {
    let cache_dir = get_cache_dir();
    let api_url_path = cache_dir.join(API_URL_FILE);
    
    if api_url_path.exists() {
        fs::remove_file(api_url_path)
            .map_err(|e| format!("无法删除 API URL: {}", e))?;
    }
    
    Ok(())
}