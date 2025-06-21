use super::*;
use futures::channel::mpsc;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::tungstenite::Error as WebSocketError;
use tokio_tungstenite::tungstenite::Message as WebSocketMessage;
use tokio_tungstenite::tungstenite::Utf8Bytes;
use tokio_tungstenite::WebSocketStream;

pub struct Connection {
    stream: BoxStream<'static, Result<String>>,
    sender: Sender,
}

impl Connection {
    pub fn new<S>(socket: WebSocketStream<S>) -> Self
    where
        S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let (mut sink, stream) = socket.split();

        // Create channel to send messages.
        let (channel, mut receiver) = mpsc::channel(0);
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match sink.send(msg).await {
                    Ok(_) => {}
                    Err(WebSocketError::ConnectionClosed) => break,
                    Err(err) => {
                        ::tracing::warn!(?err, "Connection/sink::send");
                        break;
                    }
                }
            }
        });

        // Abstract away the websocket specifics to get a stream of strings.
        let pong_channel = channel.clone();
        let stream = Box::pin(
            stream
                .err_into()
                .try_take_while(|msg| future::ready(Ok(!msg.is_close())))
                .try_filter_map(move |msg| {
                    let mut pong_sender = pong_channel.clone();
                    async move {
                        match msg {
                            WebSocketMessage::Close(_) => unreachable!(),
                            WebSocketMessage::Ping(data) => {
                                pong_sender.send(WebSocketMessage::Pong(data)).await?;
                                Ok(None)
                            }
                            WebSocketMessage::Pong(_) => Ok(None),
                            WebSocketMessage::Text(text) => Ok(Some(text.to_string())),
                            WebSocketMessage::Binary(data) => {
                                Ok(Some(String::from_utf8(data.to_vec())?))
                            }
                            WebSocketMessage::Frame(_) => Ok(None),
                        }
                    }
                }),
        );

        let sender = Sender { channel };
        Self { stream, sender }
    }

    pub async fn send<T: Serialize>(&mut self, msg: &T) -> Result<()> {
        self.sender.send(msg).await
    }

    pub fn sender(&self) -> Sender {
        self.sender.clone()
    }

    pub async fn recv<T>(&mut self) -> Option<Result<T>>
    where
        for<'de> T: Deserialize<'de>,
    {
        self.stream
            .next()
            .await
            .map(|item| item.and_then(|text| Ok(serde_json::from_str(&text)?)))
    }
}

#[derive(Clone)]
pub struct Sender {
    channel: mpsc::Sender<WebSocketMessage>,
}

impl Sender {
    pub async fn send<T: Serialize>(&mut self, msg: &T) -> Result<()> {
        self.channel
            .send(WebSocketMessage::Text(Utf8Bytes::from(
                serde_json::to_string(msg)?,
            )))
            .await?;
        Ok(())
    }
}
