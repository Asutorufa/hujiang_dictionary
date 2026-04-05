use crate::{CompletionResponse, Error};
use futures_util::Stream;
use serde::Deserialize;

#[derive(Deserialize)]
struct StreamCompletionResponse {
    response: Option<String>,
    choices: Option<Vec<Choice>>,
    thought: Option<String>,
    reasoning: Option<String>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
    reasoning_content: Option<String>,
    thought: Option<String>,
    reasoning: Option<String>,
}

pub fn parse_stream<S, E>(
    stream: S,
) -> impl Stream<Item = Result<CompletionResponse, Error>> + 'static
where
    S: Stream<Item = Result<bytes::Bytes, E>> + 'static + Unpin,
    E: std::error::Error + Send + Sync + 'static,
{
    futures_util::stream::unfold(
        (stream, String::new(), String::new()),
        |(mut stream, mut buffer, mut event_data)| async move {
            loop {
                use futures_util::StreamExt;

                if let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].to_string();
                    buffer.drain(..pos + 1);
                    let line = line.trim_end_matches('\r');

                    if line.is_empty() {
                        if !event_data.is_empty() {
                            let data = event_data.clone();
                            event_data.clear();
                            if data == "[DONE]" {
                                return None;
                            }
                            match serde_json::from_str::<StreamCompletionResponse>(&data) {
                                Ok(response) => {
                                    let mut content = response.response.unwrap_or_default();
                                    let mut thinking = response.thought.or(response.reasoning);

                                    if let Some(choice) =
                                        response.choices.and_then(|c| c.into_iter().next())
                                    {
                                        if let Some(c) = choice.delta.content {
                                            content.push_str(&c);
                                        }
                                        if let Some(t) = choice
                                            .delta
                                            .reasoning_content
                                            .or(choice.delta.reasoning)
                                            .or(choice.delta.thought)
                                        {
                                            if thinking.is_none() {
                                                thinking = Some(t);
                                            } else if let Some(think) = thinking.as_mut() {
                                                think.push_str(&t);
                                            }
                                        }
                                    }

                                    if !content.is_empty() || thinking.is_some() {
                                        return Some((
                                            Ok(CompletionResponse { content, thinking }),
                                            (stream, buffer, event_data),
                                        ));
                                    }
                                }
                                Err(e) => {
                                    return Some((
                                        Err(Error::from(e)),
                                        (stream, buffer, event_data),
                                    ));
                                }
                            }
                        }
                    } else if let Some(data) = line.strip_prefix("data: ") {
                        let data = data.trim();
                        if data == "[DONE]" {
                            if !event_data.is_empty() {
                                // If we have pending data, we try to parse it first
                                // But [DONE] usually comes after a blank line.
                                // If it doesn't, we just end here for now.
                            }
                            return None;
                        }
                        if !event_data.is_empty() {
                            event_data.push('\n');
                        }
                        event_data.push_str(data);
                    }
                    continue;
                }

                match stream.next().await {
                    Some(Ok(chunk)) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                    }
                    Some(Err(e)) => {
                        return Some((
                            Err(Error::Internal(e.to_string())),
                            (stream, buffer, event_data),
                        ));
                    }
                    None => {
                        return None;
                    }
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures_util::StreamExt;
    use futures_util::stream;

    #[tokio::test]
    async fn test_parse_stream() {
        let chunks = vec![
            Ok::<_, std::io::Error>(Bytes::from("data: {\"response\": \"Hello\"}\n\n")),
            Ok::<_, std::io::Error>(Bytes::from("data: {\"response\": \" World\"}\n\n")),
            Ok::<_, std::io::Error>(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(chunks);
        let parsed_stream = parse_stream(mock_stream);

        let results: Vec<_> = parsed_stream.collect().await;

        assert_eq!(results.len(), 2);

        let first = results[0].as_ref().unwrap();
        assert_eq!(first.content, "Hello");

        let second = results[1].as_ref().unwrap();
        assert_eq!(second.content, " World");
    }

    #[tokio::test]
    async fn test_parse_stream_fragmented() {
        let chunks = vec![
            Ok::<_, std::io::Error>(Bytes::from("data: {\"response\": ")),
            Ok::<_, std::io::Error>(Bytes::from("\"Hello\"}\n\n")),
            Ok::<_, std::io::Error>(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(chunks);
        let parsed_stream = parse_stream(mock_stream);

        let results: Vec<_> = parsed_stream.collect().await;

        assert_eq!(results.len(), 1);

        let first = results[0].as_ref().unwrap();
        assert_eq!(first.content, "Hello");
    }

    #[tokio::test]
    async fn test_parse_stream_error() {
        let chunks = vec![Ok::<_, std::io::Error>(Bytes::from(
            "data: invalid_json\n\n",
        ))];

        let mock_stream = stream::iter(chunks);
        let parsed_stream = parse_stream(mock_stream);

        let results: Vec<_> = parsed_stream.collect().await;

        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }

    #[tokio::test]
    async fn test_parse_stream_rn() {
        let chunks = vec![
            Ok::<_, std::io::Error>(Bytes::from("data: {\"response\": \"Hello\"}\r\n\r\n")),
            Ok::<_, std::io::Error>(Bytes::from("data: {\"response\": \" World\"}\r\n\r\n")),
        ];

        let mock_stream = stream::iter(chunks);
        let parsed_stream = parse_stream(mock_stream);

        let results: Vec<_> = parsed_stream.collect().await;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].as_ref().unwrap().content, "Hello");
        assert_eq!(results[1].as_ref().unwrap().content, " World");
    }

    #[tokio::test]
    async fn test_parse_stream_multiline_data() {
        let chunks = vec![Ok::<_, std::io::Error>(Bytes::from(
            "data: {\"response\":\ndata: \"Hello\"}\n\n",
        ))];

        let mock_stream = stream::iter(chunks);
        let parsed_stream = parse_stream(mock_stream);

        let results: Vec<_> = parsed_stream.collect().await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_ref().unwrap().content, "Hello");
    }

    #[tokio::test]
    async fn test_parse_stream_openai_format() {
        let chunks = vec![
            Ok::<_, std::io::Error>(Bytes::from(
                "data: {\"choices\": [{\"delta\": {\"content\": \"Hello\"}}]}\n\n",
            )),
            Ok::<_, std::io::Error>(Bytes::from(
                "data: {\"choices\": [{\"delta\": {\"reasoning_content\": \"Thinking...\"}}]}\n\n",
            )),
        ];

        let mock_stream = stream::iter(chunks);
        let parsed_stream = parse_stream(mock_stream);

        let results: Vec<_> = parsed_stream.collect().await;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].as_ref().unwrap().content, "Hello");
        assert_eq!(
            results[1].as_ref().unwrap().thinking,
            Some("Thinking...".to_string())
        );
    }

    #[tokio::test]
    async fn test_parse_stream_thought_format() {
        let chunks = vec![
            Ok::<_, std::io::Error>(Bytes::from("data: {\"thought\": \"Initial thought\"}\n\n")),
            Ok::<_, std::io::Error>(Bytes::from(
                "data: {\"choices\": [{\"delta\": {\"thought\": \" more thought\"}}]}\n\n",
            )),
            Ok::<_, std::io::Error>(Bytes::from(
                "data: {\"choices\": [{\"delta\": {\"reasoning\": \" even more\"}}]}\n\n",
            )),
            Ok::<_, std::io::Error>(Bytes::from(
                "data: {\"choices\": [{\"delta\": {\"content\": \"Result content\"}}]}\n\n",
            )),
        ];

        let mock_stream = stream::iter(chunks);
        let parsed_stream = parse_stream(mock_stream);

        let results: Vec<_> = parsed_stream.collect().await;

        assert_eq!(results.len(), 4);
        assert_eq!(
            results[0].as_ref().unwrap().thinking,
            Some("Initial thought".to_string())
        );
        assert_eq!(
            results[1].as_ref().unwrap().thinking,
            Some(" more thought".to_string())
        );
        assert_eq!(
            results[2].as_ref().unwrap().thinking,
            Some(" even more".to_string())
        );
        assert_eq!(results[3].as_ref().unwrap().content, "Result content");
    }
}
