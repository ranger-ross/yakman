use crate::api;
use chrono::prelude::DateTime;
use chrono::Utc;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, UNIX_EPOCH};
use yak_man_core::model::{ConfigInstance, ConfigInstanceChange, ConfigInstanceRevision};

#[derive(Serialize, Deserialize, Clone)]
struct PageData {
    revisions: Vec<ConfigInstanceRevision>,
    instance: ConfigInstance,
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

            let instance_metadata = api::fetch_instance_metadata(&config_name(), &instance()).await;

            PageData {
                revisions: data,
                instance: instance_metadata,
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
            .map(|d| d.instance.current_revision)
            .unwrap_or(String::from(""))
    };
    let pending_revision = move || {
        data.read(cx)
            .map(|d| d.instance.pending_revision)
            .unwrap_or(None)
    };

    let changelog = move || {
        data.read(cx)
            .map(|d| {
                let mut changelog = d.instance.changelog;
                changelog.reverse();
                changelog
            })
            .unwrap_or(vec![])
    };

    let on_select_revision = move |revision: String| {
        spawn_local(async move {
            match api::update_instance_revision(&config_name(), &instance(), &revision).await {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    let _ = navigate(
                        &format!("/apply/{}/{}", config_name(), instance()),
                        Default::default(),
                    );
                }
                Err(err) => error!("Error updating revision {}", err.to_string()),
            };
        })
    };

    view! { cx,
        <div>
            <h1>{format!("History {} -> {}", config_name(), instance())}</h1>

            <h3>{"Revisions"}</h3>

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
                                on:click={move |_| on_select_revision(rev.clone())}
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

            <h3>{"Changelog"}</h3>

            <For
                each=changelog
                key=|change| change.timestamp_ms
                view=move |cx, change: ConfigInstanceChange| view! { cx,
                    <div style="display: flex; gap: 10px">
                        <p>{format_date(change.timestamp_ms)}{" =>"}</p>
                        <p>"Previous: "{change.previous_revision}{" =>"}</p>
                        <p>"New: "{change.new_revision}</p>
                    </div>
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
