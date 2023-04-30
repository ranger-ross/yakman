use crate::api;
use leptos::*;
use leptos_router::*;

#[component]
pub fn add_config_page(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::from(""));

    let on_create_config = create_action(cx, move |name: &String| {
        let name = name.clone();

        async move {
            match api::create_config(&name).await {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    let _ = navigate("/", Default::default()); // TODO: Fix warning
                },
                Err(err) => error!("Error creating config: {}", err.to_string()),
            };
        }
    });

    view! { cx,
        <div>
            <h1>{"Add Config"}</h1>
            <div>{"Name: "} <input type="text" on:input=move |ev| set_name(event_target_value(&ev)) prop:value=name /></div>

            <br />

            <button on:click=move |_| on_create_config.dispatch(name())>"Create"</button>

        </div>
    }
}
