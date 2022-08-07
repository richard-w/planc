use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::*;

#[function_component(Home)]
pub fn home() -> Html {
    let history = use_history().unwrap();
    let context = use_context::<Rc<AppContext>>().unwrap();
    let name_ref = use_node_ref();
    let session_ref = use_node_ref();
    let on_submit = {
        let name_ref = name_ref.clone();
        let session_ref = session_ref.clone();
        Callback::from(move |event: FocusEvent| {
            event.prevent_default();
            *context.name_mut() = Some(name_ref.cast::<HtmlInputElement>().unwrap().value());
            history.push(Route::Session {
                id: session_ref.cast::<HtmlInputElement>().unwrap().value(),
            });
        })
    };
    html! {
        <form onsubmit={on_submit}>
            <label for="name">{"Your Name"}</label>
            <input type="text" id="name" ref={name_ref} />
            <label for="session">{"Session"}</label>
            <input type="text" id="session" ref={session_ref} />
            <input type="submit" value="Join" />
        </form>
    }
}
