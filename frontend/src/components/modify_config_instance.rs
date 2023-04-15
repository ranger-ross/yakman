use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::routes::Route;

use super::super::api;
use gloo_console::log;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yak_man_core::model::LabelType;
use yew::prelude::*;
use yew_router::prelude::use_navigator;

#[derive(Properties, PartialEq)]
pub struct LabelSelectionProps {
    labels: Vec<LabelType>,
    selected_labels_state: HashMap<String, Option<String>>,
    on_change: Callback<HashMap<String, Option<String>>>,
}

#[function_component(LabelSelection)]
pub fn label_selection(props: &LabelSelectionProps) -> Html {
    let selected_labels = Rc::new(RefCell::new(props.selected_labels_state.clone()));

    let on_change = Rc::new(props.on_change.clone());

    let on_select_change = Callback::from(move |event: Event| {
        let input = event.target_unchecked_into::<HtmlInputElement>(); // TODO: This sucks
        let value = input.value();
        let mut selected = selected_labels.borrow().clone();
        let old_value = selected.get_mut(&input.name()).unwrap();
        *old_value = Some(value);

        on_change.emit(selected);
    });

    html! {
        <>
            {props.labels.iter().map(|label| html! {
                <>
                    <br />
                    {&label.name}
                    <select onchange={&on_select_change} name={String::from(&label.name)}>
                        <option value="none" selected={true}>{"None"}</option>
                        {label.options.iter().map(|option| html! {
                            <option
                                value={option.clone()}
                            >{option}</option>
                        }).collect::<Html>()}
                    </select>
                </>
            }).collect::<Html>()}
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct CreateConfigInstancePageProps {
    pub config_name: String,
}

#[function_component(CreateConfigInstancePage)]
pub fn create_config_instance_page(props: &CreateConfigInstancePageProps) -> Html {
    let navigator = use_navigator().unwrap();
    let input_value_handle = use_state(String::default);
    let input_value = (*input_value_handle).clone();

    let on_change = Callback::from(move |e: Event| {
        let value = e.target_unchecked_into::<HtmlTextAreaElement>().value();
        input_value_handle.set(value);
    });

    let config_name = props.config_name.clone();

    let labels: UseStateHandle<Vec<LabelType>> = use_state(|| vec![]);
    let selected_labels_state: UseStateHandle<HashMap<String, Option<String>>> =
        use_state(HashMap::new);
    let selected_labels_state_value = (*selected_labels_state).clone();
    {
        let label_data = labels.clone();
        let selected_labels_state = selected_labels_state.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let data = api::fetch_labels().await;
                    let mut m = HashMap::new();
                    for label in &data {
                        m.insert(String::from(&label.name), None);
                    }
                    selected_labels_state.set(m);
                    label_data.set(data);
                });
            },
            (),
        );
    }

    let selected_labels_state_value_clone = (*selected_labels_state).clone();

    let on_add_clicked = move |_| {
        let config_name = config_name.clone(); // TODO: maybe handle this better?
        let input_value = input_value.clone();
        let selected_labels = selected_labels_state_value_clone.clone();
        let navigator = navigator.clone();

        log!("Selected labels len = ", selected_labels.len());

        wasm_bindgen_futures::spawn_local(async move {
            let selected_labels: HashMap<String, String> = selected_labels
                .into_iter()
                .filter_map(|(key, v)| v.map(|value| (key, value)))
                .collect();

            api::create_config_instance(&config_name, &input_value, selected_labels).await;
            navigator.push(&Route::Home);
        });
    };

    log!(
        "selected_labels_state_value = ",
        selected_labels_state_value.len()
    );

    let on_labels_changed: Callback<HashMap<String, Option<String>>> =
        Callback::from(move |data: HashMap<String, Option<String>>| {
            log!("labels changed! len = ", data.len());
            selected_labels_state.set(data);
        });

    html! {
        <div>
            <h1>{format!("Create Config Instance {}", props.config_name)}</h1>

            <h3>{"Data"}</h3>
            <textarea onchange={on_change} />

            <LabelSelection
                labels={labels.to_vec()}
                selected_labels_state={selected_labels_state_value}
                on_change={on_labels_changed}
            />


            <br />

            <button onclick={Callback::from(on_add_clicked)}>{"Add"}</button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct EditConfigInstancePageProps {
    pub config_name: String,
    pub instance: String,
}

#[function_component(EditConfigInstancePage)]
pub fn edit_config_instance_page(props: &EditConfigInstancePageProps) -> Html {
    let navigator = use_navigator().unwrap();
    let input_value_handle = use_state(String::default);
    let input_value = (*input_value_handle).clone();

    let on_change = Callback::from(move |e: Event| {
        let value = e.target_unchecked_into::<HtmlTextAreaElement>().value();
        input_value_handle.set(value);
    });

    let labels: UseStateHandle<Vec<LabelType>> = use_state(|| vec![]);
    let selected_labels_state: UseStateHandle<HashMap<String, Option<String>>> =
        use_state(HashMap::new);
    let selected_labels_state_value = (*selected_labels_state).clone();
    {
        let label_data = labels.clone();
        let selected_labels_state = selected_labels_state.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let data = api::fetch_labels().await;
                    let mut m = HashMap::new();
                    for label in &data {
                        m.insert(String::from(&label.name), None);
                    }
                    selected_labels_state.set(m);
                    label_data.set(data);
                });
            },
            (),
        );
    }

    let config_name = props.config_name.clone();
    let instance = props.instance.clone();
    let selected_labels_state_value_clone = (*selected_labels_state).clone();
    let on_add_clicked = move |_| {
        let config_name = config_name.clone(); // TODO: maybe handle this better?
        let instance = instance.clone();
        let input_value = input_value.clone();
        let selected_labels = selected_labels_state_value_clone.clone();
        let navigator = navigator.clone();

        log!("Selected labels len = ", selected_labels.len());

        wasm_bindgen_futures::spawn_local(async move {
            let selected_labels: HashMap<String, String> = selected_labels
                .into_iter()
                .filter_map(|(key, v)| v.map(|value| (key, value)))
                .collect();

            api::update_config_instance(&config_name, &instance, &input_value, selected_labels).await;
            navigator.push(&Route::Home);
        });
    };

    log!(
        "selected_labels_state_value = ",
        selected_labels_state_value.len()
    );

    let on_labels_changed: Callback<HashMap<String, Option<String>>> =
        Callback::from(move |data: HashMap<String, Option<String>>| {
            log!("labels changed! len = ", data.len());
            selected_labels_state.set(data);
        });

    html! {
        <div>
            <h1>{format!("Edit Config Instance {} -> {}", props.config_name, props.instance)}</h1>

            <h3>{"Data"}</h3>
            <textarea onchange={on_change} />

            <LabelSelection
                labels={labels.to_vec()}
                selected_labels_state={selected_labels_state_value}
                on_change={on_labels_changed}
            />

            <br />

            <button onclick={Callback::from(on_add_clicked)}>{"Update"}</button>
        </div>
    }
}
