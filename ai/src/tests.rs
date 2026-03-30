use crate::{Completion, Message, openai, openai_responses};
use futures_util::StreamExt;
use mockito::Server;

#[tokio::test]
async fn test_openai_completion_success() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-3.5-turbo-0125",
        "system_fingerprint": "fp_44709d6fcb",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello there!",
                "reasoning_content": "I am thinking..."
            },
            "logprobs": null,
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 9,
            "completion_tokens": 12,
            "total_tokens": 21
        }
    }"#;

    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let openai = openai::OpenAI {
        name: "test".to_string(),
        base_url: server.url(),
        api_key: "test_key".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        ..Default::default()
    };

    let messages = vec![Message {
        role: "user".to_string(),
        content: "Hi".to_string(),
    }];

    let response = openai.completion(messages).await.unwrap();

    assert_eq!(response.content, "Hello there!");
    assert_eq!(response.thinking, Some("I am thinking...".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_claude_completion_success() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "model": "claude-3-5-sonnet-20240620",
        "content": [
            {
                "type": "thinking",
                "thinking": "Thinking..."
            },
            {
                "type": "text",
                "text": "Hello!"
            }
        ],
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 10,
            "output_tokens": 20
        }
    }"#;

    let mock = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let claude = crate::claude::Claude {
        name: "test".to_string(),
        base_url: server.url(),
        api_key: "test_key".to_string(),
        model: "claude-3-5-sonnet".to_string(),
        ..Default::default()
    };

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "Hi".to_string(),
        },
    ];

    let response = claude.completion(messages).await.unwrap();

    assert_eq!(response.content, "Hello!");
    assert_eq!(response.thinking, Some("Thinking...".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_claude_completion_stream_success() {
    let mut server = Server::new_async().await;

    let mock_response = "data: {\"type\": \"message_start\", \"message\": {\"id\": \"msg_123\", \"type\": \"message\", \"role\": \"assistant\", \"model\": \"claude-3-5-sonnet-20240620\", \"content\": [], \"stop_reason\": null, \"stop_sequence\": null, \"usage\": {\"input_tokens\": 10, \"output_tokens\": 1}}}\n\n\
                         data: {\"type\": \"content_block_start\", \"index\": 0, \"content_block\": {\"type\": \"thinking\", \"thinking\": \"\"}}\n\n\
                         data: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"thinking_delta\", \"thinking\": \"Thin\"}}\n\n\
                         data: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"thinking_delta\", \"thinking\": \"king\"}}\n\n\
                         data: {\"type\": \"content_block_stop\", \"index\": 0}\n\n\
                         data: {\"type\": \"content_block_start\", \"index\": 1, \"content_block\": {\"type\": \"text\", \"text\": \"\"}}\n\n\
                         data: {\"type\": \"content_block_delta\", \"index\": 1, \"delta\": {\"type\": \"text_delta\", \"text\": \"Hello\"}}\n\n\
                         data: {\"type\": \"content_block_delta\", \"index\": 1, \"delta\": {\"type\": \"text_delta\", \"text\": \"!\"}}\n\n\
                         data: {\"type\": \"content_block_stop\", \"index\": 1}\n\n\
                         data: {\"type\": \"message_delta\", \"delta\": {\"stop_reason\": \"end_turn\", \"stop_sequence\": null}, \"usage\": {\"output_tokens\": 15}}\n\n\
                         data: {\"type\": \"message_stop\"}\n\n";

    let mock = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(mock_response)
        .create_async()
        .await;

    let claude = crate::claude::Claude {
        name: "test".to_string(),
        base_url: server.url(),
        api_key: "test_key".to_string(),
        model: "claude-3-5-sonnet".to_string(),
        ..Default::default()
    };

    let messages = vec![Message {
        role: "user".to_string(),
        content: "Hi".to_string(),
    }];

    let stream = claude.completion_stream(messages).await.unwrap();
    futures_util::pin_mut!(stream);

    let mut full_content = String::new();
    let mut full_thinking = String::new();

    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        full_content.push_str(&chunk.content);
        if let Some(t) = chunk.thinking {
            full_thinking.push_str(&t);
        }
    }

    assert_eq!(full_content, "Hello!");
    assert_eq!(full_thinking, "Thinking");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_openai_completion_stream_success() {
    let mut server = Server::new_async().await;

    let mock_response = "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"I\"}}]}\n\n\
                         data: {\"choices\":[{\"delta\":{\"reasoning_content\":\" am\"}}]}\n\n\
                         data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n\
                         data: {\"choices\":[{\"delta\":{\"content\":\" there\"}}]}\n\n\
                         data: [DONE]\n\n";

    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(mock_response)
        .create_async()
        .await;

    let openai = openai::OpenAI {
        name: "test".to_string(),
        base_url: server.url(),
        api_key: "test_key".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        ..Default::default()
    };

    let messages = vec![Message {
        role: "user".to_string(),
        content: "Hi".to_string(),
    }];

    let stream = openai.completion_stream(messages).await.unwrap();
    futures_util::pin_mut!(stream);

    let mut full_content = String::new();
    let mut full_thinking = String::new();

    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        full_content.push_str(&chunk.content);
        if let Some(t) = chunk.thinking {
            full_thinking.push_str(&t);
        }
    }

    assert_eq!(full_content, "Hello there");
    assert_eq!(full_thinking, "I am");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_openai_completion_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": {"message": "Invalid API key"}}"#)
        .create_async()
        .await;

    let openai = openai::OpenAI {
        name: "test".to_string(),
        base_url: server.url(),
        api_key: "invalid".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        ..Default::default()
    };

    let messages = vec![Message {
        role: "user".to_string(),
        content: "Hi".to_string(),
    }];

    let response = openai.completion(messages).await;

    assert!(response.is_err());

    match response {
        Err(crate::Error::Http(_)) => {} // Expected
        _ => panic!("Expected HTTP error"),
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_openai_responses_completion_success() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "output": [
            {
                "content": [
                    {
                        "type": "reasoning",
                        "text": "Thinking..."
                    },
                    {
                        "type": "text",
                        "text": "Hello world"
                    }
                ]
            }
        ]
    }"#;

    let mock = server
        .mock("POST", "/responses")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let responses = openai_responses::OpenAIResponses {
        base_url: server.url(),
        api_key: "test_key".to_string(),
        model: "test_model".to_string(),
        ..Default::default()
    };

    let messages = vec![Message {
        role: "user".to_string(),
        content: "Hi".to_string(),
    }];

    let response = responses.completion(messages).await.unwrap();

    assert_eq!(response.content, "Hello world");
    assert_eq!(response.thinking, Some("Thinking...".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_openai_responses_completion_stream_success() {
    let mut server = Server::new_async().await;

    let mock_response = "data: {\"output\": [{\"content\": [{\"type\": \"reasoning\", \"text\": \"Thin\"}]}]}\n\n\
                         data: {\"output\": [{\"content\": [{\"type\": \"reasoning\", \"text\": \"king\"}]}]}\n\n\
                         data: {\"output\": [{\"content\": [{\"type\": \"text\", \"text\": \"Hello\"}]}]}\n\n\
                         data: {\"output\": [{\"content\": [{\"type\": \"text\", \"text\": \" world\"}]}]}\n\n\
                         data: [DONE]\n\n";

    let mock = server
        .mock("POST", "/responses")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(mock_response)
        .create_async()
        .await;

    let responses = openai_responses::OpenAIResponses {
        base_url: server.url(),
        api_key: "test_key".to_string(),
        model: "test_model".to_string(),
        ..Default::default()
    };

    let messages = vec![Message {
        role: "user".to_string(),
        content: "Hi".to_string(),
    }];

    let stream = responses.completion_stream(messages).await.unwrap();
    futures_util::pin_mut!(stream);

    let mut full_content = String::new();
    let mut full_thinking = String::new();

    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        full_content.push_str(&chunk.content);
        if let Some(t) = chunk.thinking {
            full_thinking.push_str(&t);
        }
    }

    assert_eq!(full_content, "Hello world");
    assert_eq!(full_thinking, "Thinking");

    mock.assert_async().await;
}
