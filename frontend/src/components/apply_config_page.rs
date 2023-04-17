use crate::routes::Route;

use super::super::api;
use gloo_console::{error, log};
use yak_man_core::model::{ConfigInstance, ConfigInstanceRevision};
use yew::prelude::*;

extern crate chrono;
use chrono::prelude::DateTime;
use chrono::Utc;
use std::time::{Duration, UNIX_EPOCH};
use web_sys::window;
use yew_router::prelude::use_navigator;

fn refresh_page() {
    if let Some(window) = window() {
        let location = window.location();
        let _ = location.reload();
    }
}

#[derive(Properties, PartialEq)]
pub struct ApplyConfigPageProps {
    pub config_name: String,
    pub instance: String,
}

#[function_component(ApplyConfigPage)]
pub fn apply_config_page(props: &ApplyConfigPageProps) -> Html {
    let navigator = use_navigator().unwrap();
    let revisions: UseStateHandle<Vec<ConfigInstanceRevision>> = use_state(|| vec![]);
    let current_revision: UseStateHandle<String> = use_state(String::default); // TODO: simplify state
    let pending_revision: UseStateHandle<Option<String>> = use_state(|| None);

    {
        let revisions_data = revisions.clone();
        let pending_revision = pending_revision.clone();
        let current_revision = current_revision.clone();
        let config_name = props.config_name.clone();
        let instance = props.instance.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(data) = api::fetch_instance_revisions(&config_name, &instance).await {
                        revisions_data.set(data);
                    }

                    let metadata = api::fetch_instance_metadata(&config_name).await; // TODO: add a instance query param to avoid over fetching data

                    for inst in metadata {
                        if inst.instance == instance {
                            current_revision.set(inst.current_revision.clone());
                            pending_revision.set(inst.pending_revision.clone());
                        }
                    }
                });
            },
            (),
        );
    }

    log!("current config = ", &current_revision.to_string());
    log!("pending_revision = ", pending_revision.is_some());

    let mut sorted_revisions = revisions.to_vec();
    sorted_revisions.sort_by(|a, b| a.timestamp_ms.cmp(&b.timestamp_ms));

    let config_name_data = props.config_name.clone();
    let instance_data = props.instance.clone();
    let pending_revision_data = pending_revision.clone();

    html! {
        <div>
            <h1>{format!("Apply Config {} -> {}", props.config_name, props.instance)}</h1>

            if let Some(pending_revision) = pending_revision.as_ref() {
                <div>
                    <h3> {"Pending Revision"} </h3>
                    <p> {pending_revision} </p>
                    <p>{"TODO: Show diffs"}</p>
                    <button onclick={Callback::from(move |e| {

                        let navigator = navigator.clone();
                        let config_name_data = config_name_data.clone();
                        let instance_data = instance_data.clone();
                        let pending_revision_data = pending_revision_data.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            log!("clicked!");
                            match api::approve_instance_revision(&config_name_data, &instance_data, &pending_revision_data.as_ref().unwrap()).await {
                                Ok(()) => navigator.push(&Route::Home),
                                Err(e) => error!("Error while approving config", e.to_string()),
                            };

                        })
                    })}>{"Approve"}</button>
                </div>
            } else {
                <div>
                    {"No pending revisions"}
                </div>
            }
        </div>
    }
}

fn format_date(time: i64) -> String {
    let d = UNIX_EPOCH + Duration::from_millis(time as u64);
    let datetime = DateTime::<Utc>::from(d);
    return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
}
