use std::{fmt::Write, rc::Rc};

use gloo_net::websocket::futures::WebSocket;
use yew::prelude::*;

use super::*;
use planc_common::{ClientMessage, ServerMessage, SessionState};

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
            let sender = WebSocketProcessor::new(websocket, move |message| match message {
                ServerMessage::State(state) => remote_state.set(state),
                ServerMessage::Whoami(uid) => remote_uid.set(Some(uid)),
                ServerMessage::Error(error) => remote_error.set(Some(error)),
                ServerMessage::KeepAlive => {}
            });
            // Send whoami request
            sender.send(ClientMessage::Whoami).unwrap();
            // Send name change request
            sender.send(ClientMessage::NameChange(name)).unwrap();
            sender
        })
    };
    if (*remote_error).is_some() {
        sender.close();
    }
    html! {
        <>
            if let Some(error) = (*remote_error).clone() {
                <p><b>{"Error: "}</b>{error}</p>
            } else {
                <Participants
                    remote_state={(*remote_state).clone()}
                    remote_uid={(*remote_uid).clone()}
                    on_kick={client_message_callback(&sender, ClientMessage::KickUser)}
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

fn client_message_callback<F, T>(sender: &WebSocketProcessor, f: F) -> Callback<T>
where
    F: Fn(T) -> ClientMessage + 'static,
{
    let sender = sender.clone();
    Callback::from(move |x| {
        let message = f(x);
        sender.send(message).unwrap();
    })
}
