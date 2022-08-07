use std::rc::Rc;

use yew::prelude::*;
use yew_router::prelude::*;

mod context;
mod home;
mod session;

use self::context::*;
use self::home::*;
use self::session::*;

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
    yew::start_app::<App>();
}
