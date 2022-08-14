use std::{fmt::Write, rc::Rc};

use futures::{channel::mpsc::UnboundedSender, prelude::*};
use gloo_net::websocket::{futures::WebSocket, Message as WebSocketMessage};
use yew::prelude::*;

use super::*;
use planc_protocol::{ClientMessage, ServerMessage, SessionState};

#[derive(Debug, PartialEq, Properties)]
pub struct SessionProps {
    pub id: String,
}

#[function_component(Session)]
pub fn session(props: &SessionProps) -> Html {
    let context = use_context::<Rc<AppContext>>().unwrap();
    let history = use_history().unwrap();
    let name = if let Some(name) = context.name().clone() {
        name
    } else {
        history.push(Route::Home);
        return html! {};
    };
    let websocket_uri = {
        let mut websocket_uri = String::new();
        let location = web_sys::window().unwrap().location();
        match location.protocol().unwrap().as_ref() {
            "http:" => websocket_uri += "ws://",
            "https:" => websocket_uri += "wss://",
            _ => panic!("Unknown protocol in location"),
        }
        websocket_uri += &location.hostname().unwrap();
        if let Ok(port) = location.port() {
            write!(websocket_uri, ":{}", port).unwrap();
        }
        websocket_uri += "/api/";
        websocket_uri += &props.id;
        websocket_uri
    };
    let remote_state = use_state(SessionState::default);
    let remote_uid = use_state(|| Some(String::default()));
    let remote_error = use_state(|| None);
    let sender = {
        let remote_state = remote_state.clone();
        let remote_uid = remote_uid.clone();
        let remote_error = remote_error.clone();
        use_state(move || {
            let websocket = WebSocket::open(&websocket_uri).expect("Error opening connection");
            let (mut sink, mut stream) = websocket.split();
            wasm_bindgen_futures::spawn_local(async move {
                // Handle received messages
                while let Some(raw_message) = stream.next().await {
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
                    match message {
                        ServerMessage::State(state) => remote_state.set(state),
                        ServerMessage::Whoami(uid) => remote_uid.set(Some(uid)),
                        ServerMessage::Error(error) => remote_error.set(Some(error)),
                        ServerMessage::KeepAlive => {}
                    }
                }
            });
            let (sender, mut receiver) = futures::channel::mpsc::unbounded();
            wasm_bindgen_futures::spawn_local(async move {
                // Send messages
                while let Some(message) = receiver.next().await {
                    let text = serde_json::to_string(&message).unwrap();
                    let raw_message = WebSocketMessage::Text(text);
                    if let Err(err) = sink.send(raw_message).await {
                        log::error!("Error sending message: {}", err);
                    }
                }
            });
            {
                let mut sender = sender.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    // Send whoami request
                    sender.send(ClientMessage::Whoami).await.unwrap();
                    // Send name change request
                    sender.send(ClientMessage::NameChange(name)).await.unwrap();
                });
            }
            sender
        })
    };
    html! {
        <>
            if let Some(error) = (*remote_error).clone() {
                <p><b>{"Error: "}</b>{error}</p>
            } else {
                <Participants
                    remote_state={(*remote_state).clone()}
                    remote_uid={(*remote_uid).clone()}
                    on_kick={client_message_callback(&sender, |user_id| ClientMessage::KickUser(user_id))}
                />
                <Cards
                    on_click={client_message_callback(&sender, |card: &str| ClientMessage::SetPoints(card.to_string()))}
                />
                <Admin
                    remote_state={(*remote_state).clone()}
                    remote_uid={(*remote_uid).clone()}
                    on_claim_session={client_message_callback(&sender, |_| ClientMessage::ClaimSession)}
                    on_reset_points={client_message_callback(&sender, |_| ClientMessage::ResetPoints)}
                />
            }
        </>
    }
}

fn client_message_callback<F, T>(sender: &UnboundedSender<ClientMessage>, f: F) -> Callback<T>
where
    F: Fn(T) -> ClientMessage + 'static,
{
    let sender = sender.clone();
    Callback::from(move |x| {
        let message = f(x);
        sender.unbounded_send(message).ok();
    })
}
