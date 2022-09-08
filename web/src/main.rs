use std::rc::Rc;

use yew::prelude::*;
use yew_router::prelude::*;

mod admin;
mod cards;
mod context;
mod home;
mod participants;
mod session;
mod websocket;

use self::admin::*;
use self::cards::*;
use self::context::*;
use self::home::*;
use self::participants::*;
use self::session::*;
use self::websocket::*;

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/session/:id")]
    Session { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: &Route) -> Html {
    match route {
        Route::Home => {
            html! { <Home /> }
        }
        Route::Session { id } => {
            html! { <Session id={id.clone()} /> }
        }
        Route::NotFound => {
            html! {
                <h1>{"Not Found"}</h1>
            }
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let context = use_state(|| Rc::new(AppContext::default()));
    html! {
        <ContextProvider<Rc<AppContext>> context={(*context).clone()}>
            <BrowserRouter>
                <Switch<Route> render={Switch::render(switch)} />
            </BrowserRouter>
        </ContextProvider<Rc<AppContext>>>
    }
}

fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    yew::start_app::<App>();
}
