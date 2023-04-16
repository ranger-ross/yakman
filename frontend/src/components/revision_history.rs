use super::super::api;
use gloo_console::{error, log};
use yak_man_core::model::{ConfigInstance, ConfigInstanceRevision};
use yew::prelude::*;

extern crate chrono;
use chrono::prelude::DateTime;
use chrono::Utc;
use std::time::{Duration, UNIX_EPOCH};

#[derive(Properties, PartialEq)]
pub struct RevisionHistoryPageProps {
    pub config_name: String,
    pub instance: String,
}

#[function_component(RevisionHistoryPage)]
pub fn revision_history_page(props: &RevisionHistoryPageProps) -> Html {
    let revisions: UseStateHandle<Vec<ConfigInstanceRevision>> = use_state(|| vec![]);
    let current_revision: UseStateHandle<String> = use_state(String::default);
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

                    let mut selected_instance: Option<ConfigInstance> = None;
                    for inst in metadata {
                        if inst.instance == instance {
                            selected_instance = Some(inst);
                        }
                    }

                    if let Some(selected_instance) = selected_instance {
                        current_revision.set(selected_instance.current_revision);
                        pending_revision.set(selected_instance.pending_revision);
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
            <h1>{format!("History {} -> {}", props.config_name, props.instance)}</h1>

            <h3>{"Data"}</h3>

            {sorted_revisions.iter()
                .map(|revision| {
                    let is_current_instance = current_revision.to_string() == revision.revision;
                    let color = if is_current_instance { "yellow" } else { "cyan" };

                    let current_revision_data = current_revision.clone();
                    let rev = revision.clone();
                    let config_name = props.config_name.clone();
                    let instance = props.instance.clone();
                    html! {
                        <div style="display: flex; gap: 10px">
                            <p>{format_date(revision.timestamp_ms)}{" =>"}</p>
                            <p
                                style={format!("cursor: pointer; color: {color}")}
                                onclick={Callback::from(move |_| {
                                    log!("Clicked", &rev.revision);

                                    let rev = rev.clone();
                                    let config_name = config_name.clone();
                                    let instance = instance.clone();
                                    let current_revision_data = current_revision_data.clone();

                                    wasm_bindgen_futures::spawn_local(async move {
                                        match api::update_instance_revision(&config_name, &instance, &rev.revision).await {
                                            Ok(()) => current_revision_data.set(rev.revision),
                                            Err(err) => error!("Error updating revision", err.to_string()),
                                        };
                                    });

                                })}
                            >{&revision.revision}</p>
                        </div>
                    }
                }).collect::<Html>()}

            <br />

            if let Some(pending_revision) = pending_revision.as_ref() {
                <div>
                    <h3> {"Pending Revision"} </h3>
                    <p> {pending_revision} </p>
                    <button onclick={Callback::from(move |e| {

                        let config_name_data = config_name_data.clone();
                        let instance_data = instance_data.clone();
                        let pending_revision_data = pending_revision_data.clone();

                        wasm_bindgen_futures::spawn_local(async move {
                            log!("clicked!");
                            api::approve_instance_revision(&config_name_data, &instance_data, &pending_revision_data.as_ref().unwrap()).await; // TODO: handle error

                        })
                    })}>{"Approve"}</button>
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
