use crate::api::create_label;
use leptos::*;
use yak_man_core::model::LabelType;

#[component]
pub fn add_label_page(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::from(""));
    let (prioity, set_prioity) = create_signal(cx, String::from(""));
    let (description, set_description) = create_signal(cx, String::from(""));
    let (options, set_options) = create_signal(cx, String::from(""));

    let on_create_label = create_action(cx, |d: &(String, String, String, String)| {
        let (name, description, prioity, options) = d;

        let options = options
            .split(",")
            .into_iter()
            .map(String::from)
            .filter(|o| !o.is_empty())
            .collect::<Vec<String>>();

        let label = LabelType {
            name: name.clone(),
            description: description.clone(),
            priority: prioity.parse().unwrap(),
            options: options,
        };

        async move {
            log!("from action! {:?}", label);
            create_label(label).await.unwrap()
        }
    });

    view! { cx,
        <div>
            <h1>{"Add Label"}</h1>
            <div>{"Name: "} <input type="text" on:input=move |ev| set_name(event_target_value(&ev)) prop:value=name /></div>
            <div>{"Prioity: "} <input  type="text" on:input=move |ev| set_prioity(event_target_value(&ev)) prop:value=prioity/></div>
            <div>{"Description: "} <input  type="text" on:input=move |ev| set_description(event_target_value(&ev)) prop:value=description /></div>
            <div>{"Options: "} <input  type="text" on:input=move |ev| set_options(event_target_value(&ev)) prop:value=options /></div>

            <br />

            <button on:click=move |_| on_create_label.dispatch((name(), description(), prioity(), options()))>"Create"</button>

        </div>
    }
}
