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

    view! { cx,
        <div>
            <h1>{"Apply Config "} {config_name} {" -> "} {instance}</h1>

            {move || match page_data.read(cx) {
                Some(data) => view! { cx,
                    {move || match &data.pending_revision {
                        Some(pending_revision) => view! {cx,
                            <div>
                                <h3> {"Pending Revision"} </h3>
                                <p> {pending_revision} </p>
                                <p>{"TODO: Show diffs"}</p>
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
