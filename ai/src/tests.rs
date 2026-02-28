#[cfg(test)]
mod tests {
    use crate::{Completion, Message, openai};
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

        let mock = server.mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async().await;

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
    async fn test_openai_completion_stream_success() {
        let mut server = Server::new_async().await;

        let mock_response = "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"I\"}}]}\n\n\
                             data: {\"choices\":[{\"delta\":{\"reasoning_content\":\" am\"}}]}\n\n\
                             data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n\
                             data: {\"choices\":[{\"delta\":{\"content\":\" there\"}}]}\n\n\
                             data: [DONE]\n\n";

        let mock = server.mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(mock_response)
            .create_async().await;

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

        let mock = server.mock("POST", "/chat/completions")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": {"message": "Invalid API key"}}"#)
            .create_async().await;

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
            Err(crate::Error::Http(_)) => {}, // Expected
            _ => panic!("Expected HTTP error"),
        }

        mock.assert_async().await;
    }
}
