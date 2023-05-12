use std::os::raw;

use difference::{Changeset, Difference};
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use yak_man_core::model::ConfigInstanceRevision;

use crate::api;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct ApplyConfigPageData {
    revisions: Vec<ConfigInstanceRevision>,
    pending_revision: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct DiffData {
    original: (String, String),
    new: (String, String),
}

#[component]
pub fn apply_config_page(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    // TODO: use a better way to extract params
    let config_name = move || params.with(|params| params.get("config_name").cloned().unwrap());
    let instance = move || params.with(|params| params.get("instance").cloned().unwrap());

    let page_data = create_resource(
        cx,
        move || (config_name(), instance()),
        |(config_name, instance)| async move {
            let mut revsions: Vec<ConfigInstanceRevision> = vec![];
            let mut pending_revision: Option<String> = None;

            if let Ok(data) = api::fetch_instance_revisions(&config_name, &instance).await {
                revsions = data;
            }

            let metadata = api::fetch_config_metadata(&config_name).await; // TODO: add a instance query param to avoid over fetching data

            for inst in metadata {
                if inst.instance == instance {
                    pending_revision = inst.pending_revision;
                }
            }

            ApplyConfigPageData {
                revisions: revsions,
                pending_revision: pending_revision,
            }
        },
    );

    let pending_revision = move || page_data.read(cx).unwrap().pending_revision.unwrap();

    let on_approve = move |_| {
        spawn_local(async move {
            match api::approve_instance_revision(&config_name(), &instance(), &pending_revision())
                .await
            {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    let _ = navigate(
                        &format!("/history/{}/{}", config_name(), instance()),
                        Default::default(),
                    );
                }
                Err(e) => error!("Error while approving config: {}", e.to_string()),
            };
        })
    };

    let diffs_data = create_resource(
        cx,
        move || (config_name(), instance()),
        |(config_name, instance)| async move {
            let (data, content_type) = api::fetch_config_data(&config_name, &instance)
                .await
                .unwrap();

            DiffData {
                original: (data, content_type),
                new: (String::from("This is a placeholder diff"), String::from("text/plain")), // TODO: Actually fetch some data
            }
        },
    );

    let original_text = move || diffs_data.read(cx).map(|d| d.original.0);
    let new_text = move || diffs_data.read(cx).map(|d| d.new.0);

    view! { cx,
        <div>
            <h1>{"Apply Config "} {config_name} {" -> "} {instance}</h1>

            {original_text}

            {move || match page_data.read(cx) {
                Some(data) => view! { cx,
                    {move || match &data.pending_revision {
                        Some(pending_revision) => view! {cx,
                            <div>
                                <h3> {"Pending Revision => "} {pending_revision} </h3>
                                <ConfigDiffs
                                    original=original_text().unwrap_or("Loading".to_string())
                                    new=new_text().unwrap_or("Loading".to_string())
                                />
                                <button on:click=on_approve>{"Approve"}</button>
                            </div>
                        }.into_view(cx),
                        None => view! {cx, "No pending revisions"}.into_view(cx)
                    }}
                }.into_view(cx),
                None => view! { cx, <p>"Loading..."</p> }.into_view(cx),
            }}

        </div>
    }
}

#[derive(Debug, Clone)]
enum TextColor {
    Regular,
    Green,
    StrongGreen,
    Red,
}

impl TextColor {
    fn styles(&self) -> String {
        match self {
            TextColor::Regular => String::from(""),
            TextColor::Green => String::from("color: darkgreen"),
            TextColor::StrongGreen => String::from("color: lime"),
            TextColor::Red => String::from("color: red"),
        }
    }
}

#[component]
fn config_diffs(cx: Scope, #[prop()] original: String, #[prop()] new: String) -> impl IntoView {
    let grouped_by_lines = move || {
        let Changeset { diffs, .. } = Changeset::new(&original, &new, "\n");

        let mut grouped_by_lines: Vec<Vec<(String, TextColor)>> = vec![];

        for i in 0..diffs.len() {
            match diffs[i] {
                Difference::Same(ref x) => {
                    grouped_by_lines.push(vec![(x.clone(), TextColor::Regular)]);
                }
                Difference::Add(ref x) => {
                    let mut changes = vec![];

                    match diffs[i - 1] {
                        Difference::Rem(ref y) => {
                            let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                            for c in diffs {
                                match c {
                                    Difference::Same(ref z) => {
                                        changes.push((z.clone(), TextColor::Green));
                                    }
                                    Difference::Add(ref z) => {
                                        changes.push((z.clone(), TextColor::StrongGreen));
                                    }
                                    _ => (),
                                }
                            }
                        }
                        _ => {
                            changes.push((x.clone(), TextColor::Green));
                        }
                    };
                    grouped_by_lines.push(changes);
                }
                Difference::Rem(ref x) => {
                    grouped_by_lines.push(vec![(x.clone(), TextColor::Red)]);
                }
            }
        }

        grouped_by_lines
    };

    view! { cx,
     <div>
        {move || grouped_by_lines().into_iter().map(|line| {
            view! { cx,
                <p>
                    {move || line.iter().map(|(text, color)| {
                        view! { cx,
                            <span style={color.styles()}>{text}</span> // TODO: Handle white space better
                        }
                    })
                    .collect::<Vec<_>>()}
                </p>
            }
        })
        .collect::<Vec<_>>()}

     </div>
    }
}
