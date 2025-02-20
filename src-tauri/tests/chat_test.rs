use chat_ai_lib::chat::{ChatMessage, ChatPayload, StreamResponse, StreamChoice, DeltaContent};

#[test]
fn test_chat_message() {
    let message = ChatMessage {
        role: "user".to_string(),
        content: "Hello".to_string(),
    };
    
    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello");
}

#[test]
fn test_chat_payload() {
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }
    ];
    
    let payload = ChatPayload {
        model: "gpt-3.5-turbo".to_string(),
        messages: messages.clone(),
        stream: true,
    };
    
    assert_eq!(payload.model, "gpt-3.5-turbo");
    assert_eq!(payload.messages.len(), 1);
    assert_eq!(payload.messages[0].role, "user");
    assert_eq!(payload.messages[0].content, "Hello");
    assert!(payload.stream);
}

#[test]
fn test_stream_response() {
    let delta = DeltaContent {
        content: Some("Hello".to_string()),
    };
    
    let choice = StreamChoice {
        delta,
        finish_reason: None,
    };
    
    let response = StreamResponse {
        choices: vec![choice],
    };
    
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].delta.content, Some("Hello".to_string()));
    assert_eq!(response.choices[0].finish_reason, None);
}

#[test]
fn test_stream_response_serialization() {
    let delta = DeltaContent {
        content: Some("Hello".to_string()),
    };
    
    let choice = StreamChoice {
        delta,
        finish_reason: Some("stop".to_string()),
    };
    
    let response = StreamResponse {
        choices: vec![choice],
    };
    
    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: StreamResponse = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(response.choices.len(), deserialized.choices.len());
    assert_eq!(
        response.choices[0].delta.content,
        deserialized.choices[0].delta.content
    );
    assert_eq!(
        response.choices[0].finish_reason,
        deserialized.choices[0].finish_reason
    );
}

#[test]
fn test_chat_message_serialization() {
    let message = ChatMessage {
        role: "user".to_string(),
        content: "Hello".to_string(),
    };
    
    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: ChatMessage = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(message.role, deserialized.role);
    assert_eq!(message.content, deserialized.content);
}

#[test]
fn test_chat_payload_serialization() {
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }
    ];
    
    let payload = ChatPayload {
        model: "gpt-3.5-turbo".to_string(),
        messages: messages.clone(),
        stream: true,
    };
    
    let serialized = serde_json::to_string(&payload).unwrap();
    let deserialized: ChatPayload = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(payload.model, deserialized.model);
    assert_eq!(payload.messages.len(), deserialized.messages.len());
    assert_eq!(payload.messages[0].role, deserialized.messages[0].role);
    assert_eq!(payload.messages[0].content, deserialized.messages[0].content);
    assert_eq!(payload.stream, deserialized.stream);
}