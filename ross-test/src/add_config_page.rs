use crate::api;
use leptos::*;

#[component]
pub fn add_config_page(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::from(""));

    let add_todo = create_action(cx, |name: &String| {
        let name = name.clone();

        async move {
            match api::create_config(&name).await {
                Ok(()) => log!("TODO: navigate to home"),
                Err(err) => error!("Error creating config: {}", err.to_string()),
            };
        }
    });

    view! { cx,
        <div>
            <h1>{"Add Config"}</h1>
            <div>{"Name: "} <input type="text" on:input=move |ev| set_name(event_target_value(&ev)) prop:value=name /></div>

            <br />

            <button on:click=move |_| add_todo.dispatch(name())>"Create"</button>

        </div>
    }
}
