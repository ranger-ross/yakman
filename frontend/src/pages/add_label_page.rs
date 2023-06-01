use crate::api;
use leptos::*;
use leptos_router::use_navigate;
use yak_man_core::model::LabelType;

#[component]
pub fn add_label_page(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::from(""));
    let (prioity, set_prioity) = create_signal(cx, String::from(""));
    let (description, set_description) = create_signal(cx, String::from(""));
    let (options, set_options) = create_signal(cx, String::from(""));

    let on_create_label = move |_| {
        let options = options()
            .split(",")
            .into_iter()
            .map(String::from)
            .filter(|o| !o.is_empty())
            .collect::<Vec<String>>();

        let label = LabelType {
            name: name(),
            description: description(),
            priority: prioity().parse().unwrap(),
            options: options,
        };

        spawn_local(async move {
            match api::create_label(label).await {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    let _ = navigate("/", Default::default());
                }
                Err(err) => error!("Error creating config: {}", err.to_string()),
            };
        });
    };

    view! { cx,
        <div style="display: flex; flex-direction: column; gap: 10px;">
            <h1>{"Add Label"}</h1>
            <div>{"Name: "} <input type="text" on:input=move |ev| set_name(event_target_value(&ev)) prop:value=name /></div>
            <div>{"Prioity: "} <input  type="text" on:input=move |ev| set_prioity(event_target_value(&ev)) prop:value=prioity/></div>
            <div>{"Description: "} <input  type="text" on:input=move |ev| set_description(event_target_value(&ev)) prop:value=description /></div>
            <div>{"Options: "} <input  type="text" on:input=move |ev| set_options(event_target_value(&ev)) prop:value=options /></div>

            <div>
                <button on:click=on_create_label>"Create"</button>
            </div>

        </div>
    }
}