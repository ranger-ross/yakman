use crate::{
    components::{LabelPill, YakManButton, YakManCard}, api,
};
use chrono::prelude::DateTime;
use chrono::Utc;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, UNIX_EPOCH};
use yak_man_core::model::Label;
use yak_man_core::model::{ConfigInstance, ConfigInstanceChange, ConfigInstanceRevision};

#[derive(Serialize, Deserialize, Clone)]
struct PageData {
    revisions: Vec<ConfigInstanceRevision>,
    instance: ConfigInstance,
}

#[component]
pub fn view_config_instance_page(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    // TODO: use a better way to extract params
    let config_name = move || params.with(|params| params.get("config_name").cloned().unwrap());
    let instance = move || params.with(|params| params.get("instance").cloned().unwrap());

    let (input, set_input) = create_signal(cx, String::from(""));
    let (content_type, set_content_type) = create_signal(cx, String::from("text/plain"));
    let (selected_labels, set_selected_labels) = create_signal::<Vec<Label>>(cx, Vec::new());

    // Load previous data and pre-populate textbox/content_type with data
    spawn_local(async move {
        match api::fetch_config_data(&config_name(), &instance()).await {
            Ok((data, content_type)) => {
                set_input.set(data);
                set_content_type.set(content_type);
            }
            Err(err) => {
                error!("Error loading previous data: {}", err.to_string());
            }
        }
    });

    spawn_local(async move {
        let metadata = api::fetch_instance_metadata(&config_name(), &instance()).await;
        set_selected_labels.set(metadata.labels);
    });

    let edit_link = move || format!("/edit-instance/{}/{}", config_name(), instance());

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
        <div class="container mx-auto">
            <div class="mb-2">
                <YakManCard>
                    <div class="flex justify-between items-center">
                        <div>
                            <h1 class="text-xl font-bold">{config_name}</h1>
                            <h1 class="text-md text-gray-700">{instance}</h1>
                        </div>
                        <div>
                            <a href=edit_link>
                                <YakManButton>"Edit"</YakManButton>
                            </a>
                        </div>
                    </div>
                </YakManCard>
            </div>
            <div class="mb-2">
                <YakManCard>
                    <h1 class="text-lg font-bold mb-1">"Content"</h1>
                    <div class="mb-2">
                        <YakManCard>
                            <span class="font-bold mr-2">"Content Type"</span>
                            {content_type}
                        </YakManCard>
                    </div>
                    <div class="mb-2">
                        <YakManCard>{input}</YakManCard>
                    </div>
                </YakManCard>
            </div>
            <div class="mb-2">
                <YakManCard>
                    <h1 class="text-lg font-bold mb-1">"Labels"</h1>
                    <div class="flex flex-wrap gap-2">
                        {move || {
                            selected_labels.get()
                                .iter()
                                .map(|label| {
                                    view! { cx, <LabelPill text=format!("{}={}", & label.label_type, & label.value)/> }
                                })
                                .collect::<Vec<_>>()
                        }}
                    </div>
                </YakManCard>
            </div>
            <YakManCard>
                <h1 class="text-lg font-bold mb-1">"History"</h1>
                <h3 class="text-lg font-bold text-gray-800 mt-4">{"Revisions"}</h3>
                <For
                    each=sorted_revisions
                    key=|revision| revision.revision.bytes().len()
                    view=move |cx, revision: ConfigInstanceRevision| {
                        let is_current_instance = current_revision() == revision.revision;
                        let rev = String::from(&revision.revision);
                        view! { cx,
                            <div class="flex gap-2">
                                <p>{format_date(revision.timestamp_ms)} {" =>"}</p>
                                <p
                                    class=move || {
                                        if is_current_instance { "text-yellow-400" } else { "text-blue-600 cursor-pointer" }
                                    }
                                    on:click=move |_| on_select_revision(rev.clone())
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
                <h3 class="text-lg font-bold text-gray-800 mt-4">{"Changelog"}</h3>
                <For
                    each=changelog
                    key=|change| change.timestamp_ms
                    view=move |cx, change: ConfigInstanceChange| {
                        view! { cx,
                            <div class="flex gap-2">
                                <p>{format_date(change.timestamp_ms)} {" =>"}</p>
                                <p>"Previous: " {change.previous_revision} {" =>"}</p>
                                <p>"New: " {change.new_revision}</p>
                            </div>
                        }
                    }
                />
            </YakManCard>
        </div>
    }
}

fn format_date(time: i64) -> String {
    let d = UNIX_EPOCH + Duration::from_millis(time as u64);
    let datetime = DateTime::<Utc>::from(d);
    return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
}
