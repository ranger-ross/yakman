use crate::{
    api,
    components::{YakManButton, YakManCard, YakManInput},
    utils::input_mask::mask_lower_kebab_case,
};
use leptos::*;
use leptos_router::*;

#[component]
pub fn add_project_page(cx: Scope) -> impl IntoView {
    let name = create_rw_signal(cx, String::from(""));

    let on_create_project = move |_| {
        let name = name.get();
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
                        on:keypress=mask_lower_kebab_case
                        placeholder="my-project"
                        on:input=move |ev| name.set(event_target_value(&ev))
                        value=name
                    />
                </div>
                <YakManButton on:click=on_create_project>"Create"</YakManButton>
            </YakManCard>
        </div>
    }
}
