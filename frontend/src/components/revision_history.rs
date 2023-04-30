use crate::api;
use chrono::prelude::DateTime;
use chrono::Utc;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, UNIX_EPOCH};
use yak_man_core::model::{ConfigInstance, ConfigInstanceRevision};

#[derive(Serialize, Deserialize, Clone)]
struct PageData {
    revisions: Vec<ConfigInstanceRevision>,
    current_revision: String,
    pending_revision: Option<String>,
}

#[component]
pub fn revision_history_page(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    // TODO: use a better way to extract params
    let config_name = move || params.with(|params| params.get("config_name").cloned().unwrap());
    let instance = move || params.with(|params| params.get("instance").cloned().unwrap());

    let data = create_resource(
        cx,
        || (),
        move |_| async move {
            let data = api::fetch_instance_revisions(&config_name(), &instance())
                .await
                .unwrap_or(vec![]);

            let metadata = api::fetch_config_metadata(&config_name()).await; // TODO: add a instance query param to avoid over fetching data

            let m = api::fetch_instance_metadata(&config_name(), &instance()).await;
            log!("{m:?}");

            let mut selected_instance: Option<ConfigInstance> = None;
            for inst in metadata {
                if inst.instance == instance() {
                    selected_instance = Some(inst);
                }
            }

            let current_revision = &selected_instance
                .clone()
                .map(|i| String::from(&i.current_revision.clone()))
                .unwrap_or(String::new());
            let pending_revision: Option<String> = selected_instance
                .clone()
                .map(|i| i.pending_revision)
                .unwrap_or(None);

            PageData {
                revisions: data,
                current_revision: String::from(current_revision),
                pending_revision: pending_revision,
            }
        },
    );

    let sorted_revisions = move || {
        let mut d = data.read(cx).map(|d| d.revisions).unwrap_or(vec![]);
        d.sort_by(|a, b| a.timestamp_ms.cmp(&b.timestamp_ms));
        d
    };

    let current_revision = move || {
        data.read(cx)
            .map(|d| d.current_revision)
            .unwrap_or(String::from(""))
    };
    let pending_revision = move || data.read(cx).map(|d| d.pending_revision).unwrap_or(None);

    let on_select_revision = create_action(cx, move |revision: &String| {
        let revision = revision.clone();
        async move {
            log!("click {}", revision);
            match api::update_instance_revision(&config_name(), &instance(), &revision).await {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    let _ = navigate(
                        &format!("/apply/{}/{}", config_name(), instance()),
                        Default::default(),
                    ); // TODO: Fix warning
                }
                Err(err) => error!("Error updating revision {}", err.to_string()),
            };
        }
    });

    view! { cx,
        <div>
            <h1>{format!("History {} -> {}", config_name(), instance())}</h1>

            <h3>{"Data"}</h3>

            <For
                each=sorted_revisions
                key=|revision| revision.revision.bytes().len()
                view=move |cx, revision: ConfigInstanceRevision| {
                    let is_current_instance = current_revision() == revision.revision;
                    let color = if is_current_instance { "yellow" } else { "cyan" };
                    let rev = String::from(&revision.revision);
                    view! {cx,
                        <div style="display: flex; gap: 10px">
                            <p>{format_date(revision.timestamp_ms)}{" =>"}</p>
                            <p
                                style={format!("cursor: pointer; color: {color}")}
                                on:click={move |_| on_select_revision.dispatch(rev.clone())}
                            >
                                {&revision.revision}
                            </p>

                            <p>
                                {move || match pending_revision() {
                                    Some(rev) if rev == revision.revision => "(pending)",
                                    _ => "",
                                }}
                            </p>
                        </div>
                    }
                }
            />

            <br />

        </div>
    }
}

fn format_date(time: i64) -> String {
    let d = UNIX_EPOCH + Duration::from_millis(time as u64);
    let datetime = DateTime::<Utc>::from(d);
    return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
}
