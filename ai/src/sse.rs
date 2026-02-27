use crate::{CompletionResponse, Error};
use futures_util::Stream;
use serde::Deserialize;

#[derive(Deserialize)]
struct StreamCompletionResponse {
    response: Option<String>,
}

pub fn parse_stream<S, E>(
    stream: S,
) -> impl Stream<Item = Result<CompletionResponse, Error>> + Send
where
    S: Stream<Item = Result<bytes::Bytes, E>> + Send + 'static + Unpin,
    E: std::error::Error + Send + Sync + 'static,
{
    futures_util::stream::unfold(
        (stream, String::new()),
        |(mut stream, mut buffer)| async move {
            loop {
                use futures_util::StreamExt;

                match stream.next().await {
                    Some(Ok(chunk)) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                    }
                    Some(Err(e)) => return Some((Err(Error::Internal(e.to_string())), (stream, buffer))),
                    None => {
                        if !buffer.is_empty() {
                            let message = buffer.clone();
                            buffer.clear();
                            if let Some(data) = message.strip_prefix("data: ") {
                                let data = data.trim();
                                if !data.is_empty() && data != "[DONE]" {
                                    match serde_json::from_str::<StreamCompletionResponse>(data) {
                                        Ok(response) => {
                                            if let Some(content) = response.response {
                                                return Some((
                                                    Ok(CompletionResponse {
                                                        content,
                                                        thinking: None,
                                                    }),
                                                    (stream, buffer),
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            return Some((
                                                Err(Error::from(e)),
                                                (stream, buffer),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        return None;
                    }
                }

                if let Some(pos) = buffer.find("\n\n") {
                    let message = buffer[..pos].to_string();
                    buffer.drain(..pos + 2);

                    if let Some(data) = message.strip_prefix("data: ") {
                        let data = data.trim();
                        if data == "[DONE]" {
                            return None;
                        }
                        if !data.is_empty() {
                            match serde_json::from_str::<StreamCompletionResponse>(data) {
                                Ok(response) => {
                                    if let Some(content) = response.response {
                                        return Some((
                                            Ok(CompletionResponse {
                                                content,
                                                thinking: None,
                                            }),
                                            (stream, buffer),
                                        ));
                                    }
                                }
                                Err(e) => {
                                    return Some((Err(Error::from(e)), (stream, buffer)));
                                }
                            }
                        }
                    }
                    // Loop to process next message or fetch next chunk if incomplete
                    continue;
                }
            }
        },
    )
}
