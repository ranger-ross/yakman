use crate::{api, components::{YakManCard, YakManButton, YakManInput}};
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
        <div class="container mx-auto">
            <YakManCard>
                <h1 class="text-lg font-bold mb-4">{"Add Label"}</h1>

                <div class="mb-3">
                    <YakManInput
                        label="Name"
                        on:input=move |ev| set_name(event_target_value(&ev))
                        value=name
                        placeholder="my-label-name"
                    />
                </div>

                <div class="mb-3">
                    <YakManInput
                        label="Prioity"
                        on:input=move |ev| set_prioity(event_target_value(&ev))
                        value=prioity
                        placeholder="1"
                    />
                </div>

                <div class="mb-3">
                    <YakManInput
                        label="Description"
                        on:input=move |ev| set_description(event_target_value(&ev))
                        value=description
                        placeholder="My cool label description "
                    />
                </div>

                <div class="mb-3">
                    <YakManInput
                        label="Options"
                        on:input=move |ev| set_options(event_target_value(&ev))
                        value=options
                        placeholder="dev,prod"
                    />
                </div>

                <YakManButton on:click=on_create_label>"Create"</YakManButton>
            </YakManCard>
        </div>
    }
}
