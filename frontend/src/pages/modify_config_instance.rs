use std::{borrow::Cow, collections::HashMap};

use leptos::*;
use leptos_router::*;
use yak_man_core::model::LabelType;

use crate::{
    api,
    components::{YakManButton, YakManCard, YakManInput, YakManSelect, YakManTextArea},
};

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
            let selected_labels: HashMap<String, String> = selected_labels.get()
                .into_iter()
                .filter_map(|(key, v)| v.map(|value| (key, value)))
                .collect();

            match api::create_config_instance(
                &config_name(),
                &input.get(),
                selected_labels,
                Some(&content_type.get()),
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
            set_selected_labels.set(m);
        }
    });

    let labels = move || page_data.read(cx).unwrap_or(vec![]);

    view! { cx,
        <div class="container mx-auto">
            <YakManCard>
                <h1 class="text-lg font-bold mb-4">{"Create Config Instance"}</h1>
                <YakManTextArea
                    label="Data"
                    value=input
                    placeholder="My really cool config"
                    on:input=move |ev| set_input.set(event_target_value(&ev))
                />
                <div class="my-3">
                    <YakManInput
                        label="Content Type"
                        on:input=move |ev| set_content_type.set(event_target_value(&ev))
                        value=content_type
                        placeholder="my-config-name"
                    />
                </div>
                <LabelSelection
                    labels=Signal::derive(cx, labels)
                    selected_labels=selected_labels.into()
                    set_selected_labels=set_selected_labels
                />
                <YakManButton on:click=on_create>{"Add"}</YakManButton>
            </YakManCard>
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
            let selected_labels: HashMap<String, String> = selected_labels.get()
                .into_iter()
                .filter_map(|(key, v)| v.map(|value| (key, value)))
                .collect();

            match api::update_config_instance(
                &config_name(),
                &instance(),
                &input.get(),
                selected_labels,
                Some(&content_type.get()),
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

    let label_data = create_resource(
        cx,
        || (),
        |_| async move {
            let mut label_data = vec![];

            match api::fetch_labels().await {
                Ok(data) => label_data = data,
                Err(err) => error!("Error loading label: {}", err.to_string()),
            }

            label_data
        },
    );

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

    create_effect(cx, move |_| {
        if let Some(labels) = label_data.read(cx) {
            let mut m: HashMap<String, Option<String>> = HashMap::new();
            for label in &labels {
                m.insert(String::from(&label.name), None);
            }
            set_selected_labels.set(m);
        }
    });

    let labels = move || label_data.read(cx).unwrap_or(vec![]);

    view! { cx,
        <div class="container mx-auto">
            <YakManCard>
                <h1 class="text-lg font-bold mb-4">
                    {format!("Edit Config Instance {} -> {}", config_name(), instance())}
                </h1>
                <YakManTextArea
                    label="Data"
                    value=input
                    placeholder="My really cool config"
                    on:input=move |ev| set_input.set(event_target_value(&ev))
                />
                <div class="my-3">
                    <YakManInput
                        label="Content Type"
                        on:input=move |ev| set_content_type.set(event_target_value(&ev))
                        value=content_type
                        placeholder="my-config-name"
                    />
                </div>
                <LabelSelection
                    labels=Signal::derive(cx, labels)
                    selected_labels=selected_labels.into()
                    set_selected_labels=set_selected_labels
                />
                <YakManButton on:click=on_edit>{"Update"}</YakManButton>
            </YakManCard>
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

        let mut selected = selected_labels.get();
        let old_value = selected.get_mut(&name).unwrap();
        *old_value = Some(value);

        set_selected_labels.set(selected);
    };

    view! { cx,
        <div class="my-4">
            <h1 class="text-lg font-bold mb-1">{"Labels"}</h1>
            <div class="flex flex-col gap-2">
                <For
                    each=move || labels.get()
                    key=|l| l.name.clone()
                    view=move |cx, label: LabelType| {
                        view! { cx,
                            <YakManSelect label=Cow::Owned(label.name) on:change=on_select_change>
                                <option value="none" selected=true>
                                    {"None"}
                                </option>
                                {label
                                    .options
                                    .iter()
                                    .map(|option| {
                                        view! { cx, <option value=option>{option}</option> }
                                    })
                                    .collect::<Vec<_>>()}
                            </YakManSelect>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
pub fn ross(cx: Scope, #[prop(into)] label: MaybeSignal<Cow<'static, str>>) -> impl IntoView {
    view! { cx, <div class="w-64">{label}</div> }
}
