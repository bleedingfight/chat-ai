use std::fs;
use chat_ai_lib::cache::{
    get_cache_dir,
    encrypt_api_key,
    decrypt_data_from_file,
    encrypt_api_url,
    delete_api_key,
    delete_api_url,
    update_frequency,
    save_frequencies,
    MODEL_FREQUENCIES,
    API_KEYS_FILE,
    API_URL_FILE,
};

#[test]
fn test_cache_directory() {
    // 清理可能存在的环境变量
    std::env::remove_var("CHATAICACHE");
    
    let cache_dir = get_cache_dir();
    
    if cfg!(target_os = "windows") {
        // Windows路径测试
        let path_str = cache_dir.to_string_lossy();
        assert!(
            path_str.contains("AppData\\Local\\chat-ai") ||
            path_str.ends_with("\\chat-ai"),
            "Windows路径格式不正确: {}",
            path_str
        );
    } else {
        // Unix/Linux/macOS路径测试
        assert!(
            cache_dir.ends_with(".cache/chat-ai"),
            "Unix路径格式不正确: {}",
            cache_dir.to_string_lossy()
        );
    }
}

#[test]
fn test_cache_directory_with_env() {
    // 测试CHATAICACHE环境变量覆盖
    let test_path = if cfg!(target_os = "windows") {
        "C:\\test\\cache\\path"
    } else {
        "/test/cache/path"
    };
    std::env::set_var("CHATAICACHE", test_path);
    
    let cache_dir = get_cache_dir();
    assert_eq!(
        cache_dir.to_string_lossy(),
        test_path,
        "环境变量路径未正确应用"
    );
    
    // 清理环境变量
    std::env::remove_var("CHATAICACHE");
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_fallback_path() {
    // 清理环境变量
    std::env::remove_var("CHATAICACHE");
    std::env::remove_var("LOCALAPPDATA");
    
    // 设置USERPROFILE用于测试降级路径
    std::env::set_var("USERPROFILE", "C:\\Users\\TestUser");
    
    let cache_dir = get_cache_dir();
    assert!(
        cache_dir.to_string_lossy().ends_with("\\AppData\\Local\\chat-ai"),
        "Windows降级路径不正确: {}",
        cache_dir.to_string_lossy()
    );
    
    // 清理环境变量
    std::env::remove_var("USERPROFILE");
}

#[test]
fn test_api_key_encryption() {
    let test_key = "test-api-key-123";
    
    let cache_dir = get_cache_dir();
    let api_keys_path = cache_dir.join(API_KEYS_FILE);
    
    // 清理之前的测试数据
    let _ = delete_api_key(api_keys_path.clone());
    
    // 测试加密
    assert!(encrypt_api_key(test_key).is_ok());
    
    // 测试解密
    let decrypted = decrypt_data_from_file(api_keys_path.clone());
    assert!(decrypted.is_ok());
    assert_eq!(decrypted.unwrap(), test_key);
    
    // 清理测试数据
    assert!(delete_api_key(api_keys_path).is_ok());
}

#[test]
fn test_api_url_encryption() {
    let test_url = "https://test.api.com";
    
    let cache_dir = get_cache_dir();
    let api_url_path = cache_dir.join("data.enc");
    
    // 清理之前的测试数据
    let _ = delete_api_url(api_url_path.clone());
    
    // 测试加密
    assert!(encrypt_api_url(test_url).is_ok());
    
    // 测试解密
    let decrypted = decrypt_data_from_file(api_url_path.clone());
    assert!(decrypted.is_ok());
    assert_eq!(decrypted.unwrap(), test_url);
    
    // 清理测试数据
    assert!(delete_api_url(api_url_path).is_ok());
}

#[test]
fn test_model_frequencies() {
    let model = "gpt-4".to_string();
    
    // 测试成功调用
    update_frequency(model.clone(), true);
    {
        let frequencies = MODEL_FREQUENCIES.lock().unwrap();
        assert_eq!(*frequencies.get(&model).unwrap(), 1);
    }
    
    // 测试失败调用
    update_frequency(model.clone(), false);
    {
        let frequencies = MODEL_FREQUENCIES.lock().unwrap();
        assert_eq!(*frequencies.get(&model).unwrap(), -1);
    }
    
    // 验证频率文件路径
    let cache_dir = get_cache_dir();
    let frequency_file = cache_dir.join("frequency.json");
    
    // 测试保存频率数据
    save_frequencies(frequency_file.clone());
    assert!(frequency_file.exists());
    
    // 清理测试数据
    let _ = fs::remove_file(frequency_file);
}

#[test]
fn test_missing_api_key() {
    let cache_dir = get_cache_dir();
    let api_keys_path = cache_dir.join(API_KEYS_FILE);
    
    // 确保没有现有的 API key
    let _ = delete_api_key(api_keys_path.clone());
    
    // 测试解密不存在的 key
    let result = decrypt_data_from_file(api_keys_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("API key 未设置"));
}

#[test]
fn test_missing_api_url() {
    let cache_dir = get_cache_dir();
    let api_url_path = cache_dir.join(API_URL_FILE);
    
    // 确保没有现有的 API URL
    let _ = delete_api_url(api_url_path.clone());
    
    // 测试解密不存在的 URL
    let result = decrypt_data_from_file(api_url_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("API URL 未设置"));
}