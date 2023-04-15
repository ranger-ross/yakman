use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::super::api;
use gloo_console::log;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yak_man_core::model::{ConfigInstance, ConfigInstanceRevision, LabelType};
use yew::prelude::*;

extern crate chrono;
use chrono::prelude::DateTime;
use chrono::Utc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Properties, PartialEq)]
pub struct RevisionHistoryPageProps {
    pub config_name: String,
    pub instance: String,
}

#[function_component(RevisionHistoryPage)]
pub fn revision_history_page(props: &RevisionHistoryPageProps) -> Html {
    let revisions: UseStateHandle<Vec<ConfigInstanceRevision>> = use_state(|| vec![]);
    let current_revision: UseStateHandle<String> = use_state(String::default);

    {
        let revisions_data = revisions.clone();
        let current_revision = current_revision.clone();
        let config_name = props.config_name.clone();
        let instance = props.instance.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(data) = api::fetch_instance_revisions(&config_name, &instance).await
                    {
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
                        current_revision.set(selected_instance.current_revision)
                    }
                });
            },
            (),
        );
    }

    log!("current config = ", &current_revision.to_string());

    html! {
        <div>
            <h1>{format!("History {} -> {}", props.config_name, props.instance)}</h1>

            <h3>{"Data"}</h3>

            {revisions.iter().map(|revision| {
                let is_current_instance = current_revision.to_string() == revision.revision;
                let selected_message = if is_current_instance { "ACTIVE" } else { "" };

                html! {
                    <p>{format!("{} => {} => {} {}", format_date(revision.timestamp_ms), revision.revision, revision.data_key, selected_message)}</p>
                }
            }).collect::<Html>()}

            <br />
        </div>
    }
}

fn format_date(time: i64) -> String {
    let d = UNIX_EPOCH + Duration::from_millis(time as u64);
    let datetime = DateTime::<Utc>::from(d);
    return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
}
