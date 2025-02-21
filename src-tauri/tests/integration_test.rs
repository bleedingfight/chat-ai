use chat_ai_lib::{
    cache::{self, get_cache_dir, update_frequency, save_frequencies},
    chat::{ChatMessage, ChatPayload},
    handlers::{
        save_api_key,
        get_api_key,
        remove_api_key,
        save_api_url,
        get_api_url,
        remove_api_url,
    },
};

#[test]
fn test_complete_workflow() {
    // 1. 设置 API 凭证
    assert!(save_api_key("test-key".to_string()).is_ok());
    assert!(save_api_url("https://test.api.com".to_string()).is_ok());
    
    let cache_dir = get_cache_dir();
    let api_keys_path = cache_dir.join("api_keys.enc");
    let api_url_path = cache_dir.join("api_url.enc");
    
    // 2. 验证凭证已保存
    assert!(get_api_key(api_keys_path.clone()).is_ok());
    assert!(get_api_url(api_url_path.clone()).is_ok());
    
    // 3. 创建聊天消息
    let message = ChatMessage {
        role: "user".to_string(),
        content: "Test integration message".to_string(),
    };
    
    // 4. 创建聊天请求
    let _payload = ChatPayload {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![message],
        stream: true,
    };
    
    // 5. 更新模型使用频率
    update_frequency("gpt-3.5-turbo".to_string(), true);
    
    // 6. 清理测试数据
    assert!(remove_api_key(api_keys_path).is_ok());
    assert!(remove_api_url(api_url_path).is_ok());
}

#[test]
fn test_cache_operations() {
    // 1. 获取缓存目录
    let cache_dir = get_cache_dir();
    assert!(cache_dir.exists());
    
    // 2. 更新模型频率
    update_frequency("gpt-3.5-turbo".to_string(), true);
    update_frequency("gpt-4".to_string(), true);
    
    // 3. 保存频率数据
    let frequency_file = cache_dir.join("frequency.json");
    save_frequencies(frequency_file.clone());
    
    // 4. 验证频率文件存在
    assert!(frequency_file.exists());
}

#[test]
fn test_error_handling() {
    // 1. 测试未设置 API key 的错误处理
    let cache_dir = get_cache_dir();
    let api_keys_path = cache_dir.join("api_keys.enc");
    let api_url_path = cache_dir.join("api_url.enc");
    
    let result = get_api_key(api_keys_path);
    assert!(result.is_err());
    
    // 2. 测试未设置 API URL 的错误处理
    let result = get_api_url(api_url_path);
    assert!(result.is_err());
    
    // 3. 测试空 API key
    let result = save_api_key("".to_string());
    assert!(result.is_err());
    
    // 4. 测试空 API URL
    let result = save_api_url("".to_string());
    assert!(result.is_err());
}

#[test]
fn test_concurrent_access() {
    use std::thread;
    
    // 创建多个线程同时访问缓存
    let threads: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                // 更新频率统计
                update_frequency(format!("model-{}", i), true);
                
                let cache_dir = get_cache_dir();
                let api_keys_path = cache_dir.join("api_keys.enc");
                
                // 测试 API key 操作
                let _ = save_api_key(format!("test-key-{}", i));
                let _ = remove_api_key(api_keys_path);
            })
        })
        .collect();
    
    // 等待所有线程完成
    for thread in threads {
        thread.join().unwrap();
    }
    
    // 保存频率数据
    let cache_dir = get_cache_dir();
    let frequency_file = cache_dir.join("frequency.json");
    save_frequencies(frequency_file);
}