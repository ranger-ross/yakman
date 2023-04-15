use gloo_console::error;
use web_sys::HtmlInputElement;
use yak_man_core::model::LabelType;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{api, routes::Route};

#[function_component(AddConfigPage)]
pub fn add_config_page() -> Html {
    let navigator = use_navigator().unwrap();
    let input_value_handle = use_state(String::default);
    let input_value = (*input_value_handle).clone();

    let on_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        input_value_handle.set(value); // TODO: validate for duplicates?
    });

    let on_add_clicked = move |_| {
        let navigator = navigator.clone();
        let input_value = input_value.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::create_config(&input_value).await {
                Ok(()) => navigator.push(&Route::Home),
                Err(err) => error!("Error creating config", err.to_string()),
            };
        });
    };

    let labels: UseStateHandle<Vec<LabelType>> = use_state(|| vec![]);

    {
        let label_data = labels.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_labels().await {
                        Ok(data) => label_data.set(data),
                        Err(err) => error!("Erro loading labels", err.to_string()),
                    }
                });
            },
            (),
        );
    }

    html! {
        <div>
            <h1>{"Add Config"}</h1>

            {"Name: "} <input onchange={on_change} />

            <br />
            <br />

            <button onclick={on_add_clicked}>{"Create"}</button>
        </div>
    }
}
