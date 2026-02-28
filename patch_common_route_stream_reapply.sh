#!/bin/bash
cat << 'INNER_EOF' >> common/src/route.rs

use std::pin::Pin;
use futures_util::Stream;

pub enum UnifiedBody {
    Bytes(Vec<u8>),
    Stream(Pin<Box<dyn Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send>>),
}
INNER_EOF

sed -i 's/pub body: Vec<u8>/pub body: UnifiedBody/g' common/src/route.rs
sed -i 's/body,/body: UnifiedBody::Bytes(body),/g' common/src/route.rs
sed -i 's/body: msg.into_bytes()/body: UnifiedBody::Bytes(msg.into_bytes())/g' common/src/route.rs

cat << 'INNER_EOF' >> common/src/route.rs

impl UnifiedResponse {
    pub fn stream(stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send>>) -> Self {
        Self {
            status: 200,
            body: UnifiedBody::Stream(stream),
            headers: vec![
                ("Content-Type".to_string(), "text/event-stream".to_string()),
                ("Cache-Control".to_string(), "no-cache".to_string()),
                ("Connection".to_string(), "keep-alive".to_string()),
            ],
        }
    }
}
INNER_EOF
