use crate::api;
use leptos::*;
use leptos_router::*;
use yak_man_core::model::YakManProject;

#[component]
pub fn add_config_page(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::from(""));
    let (project_uuid, set_project_uuid) = create_signal(cx, String::from(""));

    let projects = create_resource(
        cx,
        || (),
        move |_| async move { api::fetch_projects().await.unwrap() },
    );

    let on_create_config = move |_| {
        let name = name();
        let project_uuid = project_uuid();
        spawn_local(async move {
            match api::create_config(&name, &project_uuid).await {
                Ok(()) => {
                    let navigate = use_navigate(cx);
                    let _ = navigate("/", Default::default());
                }
                Err(err) => error!("Error creating config: {}", err.to_string()),
            };
        })
    };

    let on_project_change = move |ev| {
        let value = event_target_value(&ev);

        let project = projects
            .read(cx)
            .expect("Page data should be loaded before user can change projects")
            .into_iter()
            .find(|project| project.uuid == value)
            .expect("The selected project should have been in the page data list");

        set_project_uuid.set(project.uuid);
    };

    view! { cx,
        <div style="display: flex; flex-direction: column; gap: 10px;">
            <h1>{"Add Config"}</h1>

            <div>{"Name: "} <input type="text" on:input=move |ev| set_name(event_target_value(&ev)) prop:value=name /></div>

            <div>
                {"Project: "}
                <select on:change=on_project_change>
                    <option value="">""</option>
                    {move || match projects.read(cx) {
                        Some(data) => {
                            let projects = move || data.clone();
                            view! { cx,
                                <For
                                    each=projects
                                    key=|p| p.uuid.clone()
                                    view=move |cx, project: YakManProject| view! {cx,
                                        <option value=project.uuid>{project.name}</option>
                                    }
                                />
                            }.into_view(cx)
                        },
                        None => view! { cx, }.into_view(cx)
                    }}
                </select>
            </div>

            <div>
                <button on:click=on_create_config>"Create"</button>
            </div>

        </div>
    }
}
