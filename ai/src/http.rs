use bytes::Bytes;
use futures_util::Stream;
use std::pin::Pin;

pub(crate) fn bytes_stream(
    response: reqwest::Response,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>>>> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::pin(futures_util::stream::once(
            async move { response.bytes().await },
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::pin(response.bytes_stream())
    }
}
