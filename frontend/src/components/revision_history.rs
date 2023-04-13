use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::super::api;
use gloo_console::log;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yak_man_core::model::LabelType;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RevisionHistoryPageProps {
    pub config_name: String,
    pub instance: String,
}

#[function_component(RevisionHistoryPage)]
pub fn revision_history_page(props: &RevisionHistoryPageProps) -> Html {
    let labels: UseStateHandle<Vec<LabelType>> = use_state(|| vec![]);
    let selected_labels_state: UseStateHandle<HashMap<String, Option<String>>> =
        use_state(HashMap::new);
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

    html! {
        <div>
            <h1>{format!("History {} -> {}", props.config_name, props.instance)}</h1>

            <h3>{"Data"}</h3>


            <br />
        </div>
    }
}
