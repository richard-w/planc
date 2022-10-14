use yew::prelude::*;

use planc_common::SessionState;

#[derive(Debug, PartialEq, Properties)]
pub struct ParticipantsProps {
    pub remote_state: SessionState,
    pub remote_uid: Option<String>,
    pub on_kick: Callback<String>,
}

#[function_component(Participants)]
pub fn participants(props: &ParticipantsProps) -> Html {
    let users = {
        let mut users = props
            .remote_state
            .users
            .iter()
            .map(|(user_id, user_state)| (user_id.clone(), user_state.clone()))
            .collect::<Vec<_>>();
        users.sort_by(|a, b| a.0.cmp(&b.0));
        users
    };
    let is_admin = matches!((&props.remote_uid, &props.remote_state.admin), (Some(uid), Some(admin_uid)) if uid == admin_uid);
    let user_lines = users.iter().map(|(user_id, user_state)| {
        let name = user_state.name.clone().unwrap_or_default();
        let points = {
            let points = user_state.points.clone().unwrap_or_default();
            if points != "-1" {
                points
            } else {
                "x".to_string()
            }
        };
        html! {
            <tr>
                <td>{name}</td>
                <td>{points}</td>
                if is_admin {
                    <td><button onclick={
                        let on_kick = props.on_kick.clone();
                        let user_id = user_id.clone();
                        Callback::from(move |_| on_kick.emit(user_id.clone()))
                    }>{"Kick"}</button></td>
                }
            </tr>
        }
    });
    html! {
        <table>
            <tr>
                <th>{"Name"}</th>
                <th>{"Points"}</th>
                if is_admin {
                    <th>{"Kick"}</th>
                }
            </tr>
            { for user_lines }
        </table>
    }
}
