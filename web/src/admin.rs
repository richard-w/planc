use yew::prelude::*;

#[derive(Debug, PartialEq, Properties)]
pub struct AdminProps {
    pub on_reset_points: Callback<()>,
}

#[function_component(Admin)]
pub fn admin(props: &AdminProps) -> Html {
    html! {
        <button onclick={
            let on_reset_points = props.on_reset_points.clone();
            Callback::from(move |_| {
                on_reset_points.emit(());
            })
        }>{"Reset Points"}</button>
    }
}
