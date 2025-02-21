use std::fs;
use chat_ai_lib::cache::{
    get_cache_dir,
    encrypt_api_key,
    decrypt_api_key,
    encrypt_api_url,
    decrypt_api_url,
    delete_api_key,
    delete_api_url,
    update_frequency,
    save_frequencies,
    MODEL_FREQUENCIES,
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
    
    // 清理之前的测试数据
    let _ = delete_api_key();
    
    // 测试加密
    assert!(encrypt_api_key(test_key).is_ok());
    
    // 测试解密
    let decrypted = decrypt_api_key();
    assert!(decrypted.is_ok());
    assert_eq!(decrypted.unwrap(), test_key);
    
    // 清理测试数据
    assert!(delete_api_key().is_ok());
}

#[test]
fn test_api_url_encryption() {
    let test_url = "https://test.api.com";
    
    // 清理之前的测试数据
    let _ = delete_api_url();
    
    // 测试加密
    assert!(encrypt_api_url(test_url).is_ok());
    
    // 测试解密
    let decrypted = decrypt_api_url();
    assert!(decrypted.is_ok());
    assert_eq!(decrypted.unwrap(), test_url);
    
    // 清理测试数据
    assert!(delete_api_url().is_ok());
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
    
    // 测试保存频率数据
    save_frequencies();
    
    // 验证频率文件是否创建
    let cache_dir = get_cache_dir();
    let frequency_file = cache_dir.join("frequency.json");
    assert!(frequency_file.exists());
    
    // 清理测试数据
    let _ = fs::remove_file(frequency_file);
}

#[test]
fn test_missing_api_key() {
    // 确保没有现有的 API key
    let _ = delete_api_key();
    
    // 测试解密不存在的 key
    let result = decrypt_api_key();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("API key 未设置"));
}

#[test]
fn test_missing_api_url() {
    // 确保没有现有的 API URL
    let _ = delete_api_url();
    
    // 测试解密不存在的 URL
    let result = decrypt_api_url();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("API URL 未设置"));
}