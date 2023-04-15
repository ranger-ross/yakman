use gloo_console::error;
use web_sys::HtmlInputElement;
use yak_man_core::model::LabelType;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{api, routes::Route};

#[function_component(AddLabelPage)]
pub fn add_label_page() -> Html {
    let navigator = use_navigator().unwrap();
    let name = use_state(String::default);
    let name_value = (*name).clone();
    let prioity = use_state(String::default);
    let prioity_value = (*prioity).clone();
    let description = use_state(String::default);
    let description_value = (*description).clone();
    let options = use_state(String::default);
    let options_value = (*options).clone();

    let on_name_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        name.set(value); // TODO: validate for duplicates?
    });

    let on_prioity_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        prioity.set(value); // TODO: validate
    });

    let on_description_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        description.set(value);
    });

    let on_options_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        options.set(value); // TODO: validate
    });

    let on_add_clicked = move |_| {
        let name = name_value.clone();
        let prioity = prioity_value.clone();
        let description = description_value.clone();
        let navigator = navigator.clone();
        let options = options_value
            .split(",")
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
        wasm_bindgen_futures::spawn_local(async move {
            match api::create_label(LabelType {
                name: name,
                description: description,
                priority: prioity.parse().unwrap(),
                options: options,
            })
            .await
            {
                Ok(()) => {}
                Err(err) => error!("Error creating label", err.to_string()),
            };
            navigator.push(&Route::Home);
        });
    };

    html! {
        <div>
            <h1>{"Add Label"}</h1>
            <div>{"Name: "} <input onchange={on_name_change} /></div>
            <div>{"Prioity: "} <input onchange={on_prioity_change} /></div>
            <div>{"Description: "} <input onchange={on_description_change} /></div>
            <div>{"Options: "} <input onchange={on_options_change} /></div>

            <br />

            <button onclick={on_add_clicked}>{"Create"}</button>
        </div>
    }
}
