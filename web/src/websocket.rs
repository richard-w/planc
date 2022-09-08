use std::{cell::Cell, rc::Rc};

use futures::{
    channel::mpsc,
    channel::oneshot,
    prelude::*,
    stream::{SplitSink, SplitStream},
};
use gloo_net::websocket::{futures::WebSocket, Message as WebSocketMessage};

use planc_protocol::{ClientMessage, ServerMessage};

struct CloseHandles {
    eventual_sink: oneshot::Receiver<SplitSink<WebSocket, WebSocketMessage>>,
    eventual_stream: oneshot::Receiver<SplitStream<WebSocket>>,
    cancel_read_sender: oneshot::Sender<()>,
}

#[derive(Clone)]
pub struct WebSocketProcessor {
    close_handles: Rc<Cell<Option<CloseHandles>>>,
    message_sender: mpsc::UnboundedSender<ClientMessage>,
}

impl WebSocketProcessor {
    pub fn new<F>(websocket: WebSocket, mut f: F) -> Self
    where
        F: FnMut(ServerMessage) + 'static,
    {
        let (sink, stream) = websocket.split();
        let (eventual_sink_provider, eventual_sink) = oneshot::channel();
        let (eventual_stream_provider, eventual_stream) = oneshot::channel();
        let (cancel_read_sender, cancel_read_receiver) = oneshot::channel();
        let (message_sender, message_receiver) = mpsc::unbounded();
        let close_handles = Rc::new(Cell::new(Some(CloseHandles {
            eventual_sink,
            eventual_stream,
            cancel_read_sender,
        })));

        // Receiver task
        wasm_bindgen_futures::spawn_local(async move {
            let mut stream = stream.fuse();
            let mut cancel_read_receiver = cancel_read_receiver;
            while let Some(raw_message) =
                futures::select! { next = stream.next() => next, _ = cancel_read_receiver => None }
            {
                let text = match raw_message {
                    Ok(WebSocketMessage::Text(text)) => text,
                    Ok(_) => {
                        log::warn!("Invalid message received");
                        continue;
                    }
                    Err(err) => {
                        log::error!("Error received message: {}", err);
                        continue;
                    }
                };
                let message = match serde_json::from_str(&text) {
                    Ok(message) => message,
                    Err(err) => {
                        log::error!("Error decoding received message: {}", err);
                        continue;
                    }
                };
                f(message);
            }
            let stream = stream.into_inner();
            eventual_stream_provider.send(stream).ok();
        });

        // Sender task
        wasm_bindgen_futures::spawn_local(async move {
            let mut sink = sink;
            let mut message_receiver = message_receiver;
            while let Some(message) = message_receiver.next().await {
                let text = serde_json::to_string(&message).unwrap();
                let raw_message = WebSocketMessage::Text(text);
                if let Err(err) = sink.send(raw_message).await {
                    log::error!("Error sending message: {}", err);
                }
            }
            eventual_sink_provider.send(sink).ok();
        });

        Self {
            close_handles,
            message_sender,
        }
    }

    pub fn send(&self, message: ClientMessage) -> anyhow::Result<()> {
        self.message_sender
            .unbounded_send(message)
            .map_err(|_| anyhow::Error::msg("Send failed"))
    }

    pub fn close(&self) {
        if let Some(close_handles) = self.close_handles.take() {
            // Cancel the write task.
            self.message_sender.close_channel();
            // Send cancellation signal to read task
            close_handles.cancel_read_sender.send(()).unwrap();
            wasm_bindgen_futures::spawn_local(async move {
                let sink = close_handles.eventual_sink.await.unwrap();
                let stream = close_handles.eventual_stream.await.unwrap();
                let websocket = sink.reunite(stream).unwrap();
                websocket.close(None, None).unwrap();
            });
        }
    }
}
