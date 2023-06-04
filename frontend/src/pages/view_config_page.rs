use crate::{
    api,
    components::{LabelPill, YakManCard, YakManButton},
};
use leptos::*;
use leptos_router::*;
use yak_man_core::model::Label;

#[component]
pub fn view_config_instance_page(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    // TODO: use a better way to extract params
    let config_name = move || params.with(|params| params.get("config_name").cloned().unwrap());
    let instance = move || params.with(|params| params.get("instance").cloned().unwrap());

    let (input, set_input) = create_signal(cx, String::from(""));
    let (content_type, set_content_type) = create_signal(cx, String::from("text/plain"));
    let (selected_labels, set_selected_labels) = create_signal::<Vec<Label>>(cx, Vec::new());

    // Load previous data and pre-populate textbox/content_type with data
    spawn_local(async move {
        match api::fetch_config_data(&config_name(), &instance()).await {
            Ok((data, content_type)) => {
                set_input.set(data);
                set_content_type.set(content_type);
            }
            Err(err) => {
                error!("Error loading previous data: {}", err.to_string());
            }
        }
    });

    spawn_local(async move {
        let metadata = api::fetch_instance_metadata(&config_name(), &instance()).await;
        set_selected_labels.set(metadata.labels);
    });

    let edit_link = move || format!("/edit-instance/{}/{}", config_name(), instance());

    view! { cx,
        <div class="container mx-auto">
            <div class="mb-2">
                <YakManCard>
                    <div class="flex justify-between items-center">
                        <div>
                            <h1 class="text-xl font-bold">{config_name}</h1>
                            <h1 class="text-md text-gray-700">{instance}</h1>
                        </div>
                        <div>
                            <a href=edit_link>
                                <YakManButton>"Edit"</YakManButton>
                            </a>
                        </div>
                    </div>
                    
                </YakManCard>
            </div>

            <div class="mb-2">
                <YakManCard>
                    <h1 class="text-lg font-bold mb-1">"Content"</h1>
                    <div class="mb-2">
                        <YakManCard>
                            <span class="font-bold mr-2">"Content Type"</span>
                            {content_type}
                        </YakManCard>
                    </div>
                    <div class="mb-2">
                        <YakManCard>{input}</YakManCard>
                    </div>
                </YakManCard>
            </div>

            <div class="mb-2">
                <YakManCard>
                    <h1 class="text-lg font-bold mb-1">"Labels"</h1>
                    <div class="flex flex-wrap gap-2">
                        {move || selected_labels().iter().map(|label| view! { cx,
                            <LabelPill text={format!("{}={}", &label.label_type, &label.value)} />
                        }).collect::<Vec<_>>()}
                    </div>
                </YakManCard>
            </div>

        </div>
    }
}
