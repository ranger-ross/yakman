use crate::api;
use leptos::*;
use leptos_router::*;

// extern crate chrono;
use chrono::prelude::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::{Duration, UNIX_EPOCH};
use yak_man_core::model::{ConfigInstance, ConfigInstanceRevision};
// use web_sys::window;

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

            let metadata = api::fetch_instance_metadata(&config_name()).await; // TODO: add a instance query param to avoid over fetching data

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

    // let on_create_config = create_action(cx, |name: &String| {
    //     let name = name.clone();

    //     async move {
    //         match api::create_config(&name).await {
    //             Ok(()) => log!("TODO: navigate to home"),
    //             Err(err) => error!("Error creating config: {}", err.to_string()),
    //         };
    //     }
    // });

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

    view! { cx,
        <div>
            <h1>{format!("History {} -> {}", config_name(), instance())}</h1>

            <h3>{"Data"}</h3>

            {move || sorted_revisions().into_iter()
                .map(|revision| {
                    let is_current_instance = current_revision().to_string() == revision.revision;
                    let color = if is_current_instance { "yellow" } else { "cyan" };

                    // let rev = revision.clone();
                    // let config_name = props.config_name.clone();
                    // let instance = props.instance.clone();
                    // let navigator = navigator.clone();
                    view! { cx,
                        <div style="display: flex; gap: 10px">
                            <p>{format_date(revision.timestamp_ms)}{" =>"}</p>
                            <p
                                style={format!("cursor: pointer; color: {color}")}
                                // onclick={Callback::from(move |_| {
                                //     log!("Clicked {}", &rev.revision);

                                //     let navigator = navigator.clone();
                                //     let rev = rev.clone();
                                //     let config_name = config_name.clone();
                                //     let instance = instance.clone();
                                //     wasm_bindgen_futures::spawn_local(async move {
                                //         match api::update_instance_revision(&config_name, &instance, &rev.revision).await {
                                //             Ok(()) => navigator.push(&Route::ApplyConfigPage {config_name: config_name, instance: instance}),
                                //             Err(err) => error!("Error updating revision", err.to_string()),
                                //         };
                                //     });

                                // })}
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
                }).collect::<Vec<_>>()}

            <br />

        </div>
    }
}

// fn refresh_page() {
//     if let Some(window) = window() {
//         let location = window.location();
//         let _ = location.reload();
//     }
// }

// #[derive(Properties, PartialEq)]
// pub struct RevisionHistoryPageProps {
//     pub config_name: String,
//     pub instance: String,
// }

// #[function_component(RevisionHistoryPage)]
// pub fn revision_history_page(props: &RevisionHistoryPageProps) -> Html {
//     let navigator = use_navigator().unwrap();
//     let revisions: UseStateHandle<Vec<ConfigInstanceRevision>> = use_state(|| vec![]);
//     let current_revision: UseStateHandle<String> = use_state(String::default);
//     let pending_revision: UseStateHandle<Option<String>> = use_state(|| None);

//     {
//         let revisions_data = revisions.clone();
//         let pending_revision = pending_revision.clone();
//         let current_revision = current_revision.clone();
//         let config_name = props.config_name.clone();
//         let instance = props.instance.clone();
//         use_effect_with_deps(
//             move |_| {
//                 wasm_bindgen_futures::spawn_local(async move {
//                     if let Ok(data) = api::fetch_instance_revisions(&config_name, &instance).await {
//                         revisions_data.set(data);
//                     }

//                     let metadata = api::fetch_instance_metadata(&config_name).await; // TODO: add a instance query param to avoid over fetching data

//                     let mut selected_instance: Option<ConfigInstance> = None;
//                     for inst in metadata {
//                         if inst.instance == instance {
//                             selected_instance = Some(inst);
//                         }
//                     }

//                     if let Some(selected_instance) = selected_instance {
//                         current_revision.set(selected_instance.current_revision);
//                         pending_revision.set(selected_instance.pending_revision);
//                     }
//                 });
//             },
//             (),
//         );
//     }

//     log!("current config = ", &current_revision.to_string());
//     log!("pending_revision = ", pending_revision.is_some());

//     let mut sorted_revisions = revisions.to_vec();
//     sorted_revisions.sort_by(|a, b| a.timestamp_ms.cmp(&b.timestamp_ms));

//     html! {
//         <div>
//             <h1>{format!("History {} -> {}", props.config_name, props.instance)}</h1>

//             <h3>{"Data"}</h3>

//             {sorted_revisions.iter()
//                 .map(|revision| {
//                     let is_current_instance = current_revision.to_string() == revision.revision;
//                     let color = if is_current_instance { "yellow" } else { "cyan" };

//                     let rev = revision.clone();
//                     let config_name = props.config_name.clone();
//                     let instance = props.instance.clone();
//                     let navigator = navigator.clone();
//                     html! {
//                         <div style="display: flex; gap: 10px">
//                             <p>{format_date(revision.timestamp_ms)}{" =>"}</p>
//                             <p
//                                 style={format!("cursor: pointer; color: {color}")}
//                                 onclick={Callback::from(move |_| {
//                                     log!("Clicked", &rev.revision);

//                                     let navigator = navigator.clone();
//                                     let rev = rev.clone();
//                                     let config_name = config_name.clone();
//                                     let instance = instance.clone();
//                                     wasm_bindgen_futures::spawn_local(async move {
//                                         match api::update_instance_revision(&config_name, &instance, &rev.revision).await {
//                                             Ok(()) => navigator.push(&Route::ApplyConfigPage {config_name: config_name, instance: instance}),
//                                             Err(err) => error!("Error updating revision", err.to_string()),
//                                         };
//                                     });

//                                 })}
//                             >
//                                 {&revision.revision}
//                             </p>
//                             if pending_revision.is_some() && pending_revision.as_ref().unwrap() == &revision.revision {
//                                 <p>
//                                     {"(pending)"}
//                                 </p>
//                             }
//                         </div>
//                     }
//                 }).collect::<Html>()}

//             <br />

//         </div>
//     }
// }

fn format_date(time: i64) -> String {
    let d = UNIX_EPOCH + Duration::from_millis(time as u64);
    let datetime = DateTime::<Utc>::from(d);
    return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
}
