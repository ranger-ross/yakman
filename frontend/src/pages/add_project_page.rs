use crate::api;
use leptos::*;
use leptos_router::*;

#[component]
pub fn add_project_page(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::from(""));

    let on_create_project = move |_| {
        let name = name();
        spawn_local(async move {
            match api::create_project(&name).await {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    let _ = navigate("/", Default::default());
                }
                Err(err) => error!("Error creating project: {}", err.to_string()),
            };
        })
    };

    view! { cx,
        <div>
            <h1>{"Add Project"}</h1>
            <div>{"Name: "} <input type="text" on:input=move |ev| set_name(event_target_value(&ev)) prop:value=name /></div>

            <br />

            <button on:click=on_create_project>"Create"</button>

        </div>
    }
}
