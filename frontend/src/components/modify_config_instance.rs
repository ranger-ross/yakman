use std::collections::HashMap;

use leptos::*;
use leptos_router::*;
use yak_man_core::model::LabelType;

use crate::api;

#[component]
pub fn create_config_instance_page(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    // TODO: use a better way to extract params
    let config_name = move || params.with(|params| params.get("config_name").cloned().unwrap());

    let (input, set_input) = create_signal(cx, String::from(""));
    let (content_type, set_content_type) = create_signal(cx, String::from("text/plain"));
    let (selected_labels, set_selected_labels) =
        create_signal::<HashMap<String, Option<String>>>(cx, HashMap::new());

    let on_create = move |_| {
        spawn_local(async move {
            let selected_labels: HashMap<String, String> = selected_labels()
                .into_iter()
                .filter_map(|(key, v)| v.map(|value| (key, value)))
                .collect();

            match api::create_config_instance(
                &config_name(),
                &input(),
                selected_labels,
                Some(&content_type()),
            )
            .await
            {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    _ = navigate("/", Default::default());
                }
                Err(err) => error!("Error creating config instance {}", err.to_string()),
            };
        })
    };

    let page_data = create_resource(
        cx,
        || (),
        |_| async move {
            log!("fetching");
            let mut label_data = vec![];

            match api::fetch_labels().await {
                Ok(data) => label_data = data,
                Err(err) => error!("Error loading label: {}", err.to_string()),
            }

            label_data
        },
    );

    create_effect(cx, move |_| {
        if let Some(labels) = page_data.read(cx) {
            let mut m: HashMap<String, Option<String>> = HashMap::new();
            for label in &labels {
                m.insert(String::from(&label.name), None);
            }
            set_selected_labels(m);
        }
    });

    let labels = move || page_data.read(cx).unwrap_or(vec![]);

    view! { cx,
        <div>
            <h1>"Create Config Instance "{config_name}</h1>

            <h3>{"Data"}</h3>
            <textarea on:input=move |ev| set_input(event_target_value(&ev)) prop:value=input />

            <LabelSelection
                labels={Signal::derive(cx, labels)}
                selected_labels={selected_labels.into()}
                set_selected_labels={set_selected_labels}
             />

            <br />

            {"Content Type "} <input on:input=move |ev| set_content_type(event_target_value(&ev)) prop:value=content_type />

            <br />

            <button on:click=on_create>{"Add"}</button>
        </div>
    }
}

#[component]
pub fn edit_config_instance_page(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    // TODO: use a better way to extract params
    let config_name = move || params.with(|params| params.get("config_name").cloned().unwrap());
    let instance = move || params.with(|params| params.get("instance").cloned().unwrap());

    let (input, set_input) = create_signal(cx, String::from(""));
    let (content_type, set_content_type) = create_signal(cx, String::from("text/plain"));
    let (selected_labels, set_selected_labels) =
        create_signal::<HashMap<String, Option<String>>>(cx, HashMap::new());

    let on_edit = move |_| {
        spawn_local(async move {
            let selected_labels: HashMap<String, String> = selected_labels()
                .into_iter()
                .filter_map(|(key, v)| v.map(|value| (key, value)))
                .collect();

            match api::update_config_instance(
                &config_name(),
                &instance(),
                &input(),
                selected_labels,
                Some(&content_type()),
            )
            .await
            {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    _ = navigate("/", Default::default());
                }
                Err(err) => error!("Error updating config instance {}", err.to_string()),
            };
        })
    };

    let page_data = create_resource(
        cx,
        || (),
        |_| async move {
            log!("fetching");
            let mut label_data = vec![];

            match api::fetch_labels().await {
                Ok(data) => label_data = data,
                Err(err) => error!("Error loading label: {}", err.to_string()),
            }

            label_data
        },
    );

    create_effect(cx, move |_| {
        if let Some(labels) = page_data.read(cx) {
            let mut m: HashMap<String, Option<String>> = HashMap::new();
            for label in &labels {
                m.insert(String::from(&label.name), None);
            }
            set_selected_labels(m);
        }
    });

    let labels = move || page_data.read(cx).unwrap_or(vec![]);

    view! { cx,
        <div>
            <h1>{format!("Edit Config Instance {} -> {}", config_name(), instance())}</h1>

            <h3>{"Data"}</h3>
            <textarea on:input=move |ev| set_input(event_target_value(&ev)) prop:value=input />

            <LabelSelection
                labels={Signal::derive(cx, labels)}
                selected_labels={selected_labels.into()}
                set_selected_labels={set_selected_labels}
             />

            <br />

            {"Content Type "} <input on:input=move |ev| set_content_type(event_target_value(&ev)) prop:value=content_type />

            <br />

            <button on:click=on_edit>{"Update"}</button>
        </div>
    }
}

#[component]
pub fn label_selection(
    cx: Scope,
    #[prop()] labels: Signal<Vec<LabelType>>,
    #[prop()] selected_labels: Signal<HashMap<String, Option<String>>>,
    #[prop()] set_selected_labels: WriteSignal<HashMap<String, Option<String>>>,
) -> impl IntoView {
    let on_select_change = move |ev| {
        let el: web_sys::HtmlSelectElement = event_target(&ev);
        let name = el.name();
        let value = event_target_value(&ev);

        let mut selected = selected_labels();
        let old_value = selected.get_mut(&name).unwrap();
        *old_value = Some(value);

        set_selected_labels(selected);
    };

    view! { cx,
        <>
        {move || labels().iter().map(|label| view! { cx,
            <>
                <br />
                {&label.name}
                <select name={String::from(&label.name)} on:change=on_select_change>
                    <option value="none" selected={true}>{"None"}</option>
                    {label.options.iter().map(|option| view! { cx,
                        <option value={option}>
                            {option}
                        </option>
                    }).collect::<Vec<_>>()}
                </select>
            </>
        }).collect::<Vec<_>>()}
    </>
    }
}
