use yew::prelude::*;

const CARDS: &[&str] = &[
    "0", "1", "2", "3", "5", "8", "13", "20", "40", "60", "100", "?", "☕",
];

#[derive(Debug, PartialEq, Properties)]
pub struct CardsProps {
    pub on_click: Callback<&'static str>,
}

#[function_component(Cards)]
pub fn cards(props: &CardsProps) -> Html {
    let cards = CARDS.iter().map(|card| {
        let on_click = {
            let callback = props.on_click.clone();
            Callback::from(move |_| {
                callback.emit(card);
            })
        };
        html! {
            <button onclick={on_click}>{card}</button>
        }
    });
    html! {
        { for cards }
    }
}
