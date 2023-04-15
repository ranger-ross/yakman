use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::super::api;
use gloo_console::log;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yak_man_core::model::{ConfigInstanceRevision, LabelType};
use yew::prelude::*;

extern crate chrono;
use chrono::prelude::DateTime;
use chrono::Utc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Properties, PartialEq)]
pub struct RevisionHistoryPageProps {
    pub config_name: String,
    pub instance: String,
}

#[function_component(RevisionHistoryPage)]
pub fn revision_history_page(props: &RevisionHistoryPageProps) -> Html {
    let revisions: UseStateHandle<Vec<ConfigInstanceRevision>> = use_state(|| vec![]);

    {
        let revisions_data = revisions.clone();
        let config_name = props.config_name.clone();
        let instance = props.instance.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(data) = api::fetch_instance_revisions(&config_name, &instance).await
                    {
                        revisions_data.set(data);
                    }
                });
            },
            (),
        );
    }

    html! {
        <div>
            <h1>{format!("History {} -> {}", props.config_name, props.instance)}</h1>

            <h3>{"Data"}</h3>

            {revisions.iter().map(|revision| html! {
                <p>{format!("{} => {} => {}", format_date(revision.timestamp_ms), revision.revision, revision.data_key)}</p>
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