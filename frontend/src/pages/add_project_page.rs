use crate::{api, components::{YakManCard, YakManInput, YakManButton}};
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
        <div class="container mx-auto">
            <YakManCard>
                <h1 class="text-lg font-bold mb-4">{"Add Project"}</h1>

                <div class="mb-3">
                    <YakManInput
                        label="Name"
                        placeholder="my-project"
                        on:input=move |ev| set_name(event_target_value(&ev))
                        value=name
                    />
                </div>

                <YakManButton on:click=on_create_project>"Create"</YakManButton>

            </YakManCard>
        </div>
    }
}
