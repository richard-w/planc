use yew::prelude::*;

use super::*;

#[derive(Debug, PartialEq, Properties)]
pub struct SessionProps {
    pub id: String,
}

#[function_component(Session)]
pub fn session(props: &SessionProps) -> Html {
    let context = use_context::<Rc<AppContext>>().unwrap();
    let history = use_history().unwrap();
    let name = context.name().clone().unwrap_or_else(|| {
        history.push(Route::Home);
        String::default()
    });
    html! {
        <>
            <p>{format!("Session ID: {}", props.id)}</p>
            <p>{format!("User Name: {}", name)}</p>
        </>
    }
}
