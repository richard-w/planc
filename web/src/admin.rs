use yew::prelude::*;

use planc_common::SessionState;

#[derive(Debug, PartialEq, Properties)]
pub struct AdminProps {
    pub remote_state: SessionState,
    pub remote_uid: Option<String>,
    pub on_claim_session: Callback<()>,
    pub on_reset_points: Callback<()>,
}

#[function_component(Admin)]
pub fn admin(props: &AdminProps) -> Html {
    let has_admin = props.remote_state.admin.is_some();
    let is_admin = matches!((&props.remote_uid, &props.remote_state.admin), (Some(uid), Some(admin_uid)) if uid == admin_uid);
    html! {
        if !has_admin {
            <button onclick={
                let on_claim_session = props.on_claim_session.clone();
                Callback::from(move |_| {
                    on_claim_session.emit(());
                })
            }>{"Claim Session"}</button>
        }
        else if is_admin {
            <button onclick={
                let on_reset_points = props.on_reset_points.clone();
                Callback::from(move |_| {
                    on_reset_points.emit(());
                })
            }>{"Reset Points"}</button>
        }
    }
}
