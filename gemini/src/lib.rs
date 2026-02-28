pub mod client;
pub mod model;

pub use client::Client;
pub use model::*;

#[cfg(test)]
mod tests {
    use crate::model::*;

    #[test]
    fn test_serialization() {
        let req = GenerateContentRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: Some("hello".to_string()),
                    inline_data: None,
                    thought: Some(false),
                }],
            }],
            tools: None,
            safety_settings: None,
            system_instruction: None,
            generation_config: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        println!("{}", json);
        assert!(json.contains("hello"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"
        {
          "candidates": [
            {
              "content": {
                "role": "model",
                "parts": [
                  {
                    "text": "Hello there!"
                  }
                ]
              },
              "finishReason": "STOP",
              "index": 0,
              "safetyRatings": [
                {
                  "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                  "probability": "NEGLIGIBLE"
                }
              ]
            }
          ]
        }
        "#;
        let resp: GenerateContentResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.candidates.len(), 1);
        assert_eq!(
            resp.candidates[0].content.parts[0].text.as_deref(),
            Some("Hello there!")
        );
    }
}
