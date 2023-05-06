use crate::api;
use leptos::*;
use serde::{Deserialize, Serialize};
use yak_man_core::model::{Config, ConfigInstance, YakManProject};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PageData {
    pub projects: Vec<YakManProject>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    pub config: Config,
    pub instances: Vec<ConfigInstance>,
}

#[component]
pub fn config_list_page(cx: Scope) -> impl IntoView {
    let (selected_project_index, set_selected_project_index) = create_signal::<usize>(cx, 0);

    let pd = create_resource(
        cx,
        || (),
        |_| async move {
            let projects = api::fetch_projects().await.unwrap();

            PageData { projects: projects }
        },
    );

    let selected_project = move || {
        pd.read(cx)
            .map(|data| data.projects[selected_project_index()].clone())
    };

    let page_data = create_resource(
        cx,
        || (),
        move |_| async move {
            log!("{:?}", pd.read(cx));
            log!("{:?}", selected_project());

            let mut configs_list: Vec<PageConfig> = vec![];

            match api::fetch_configs().await {
                Ok(configs) => {
                    for config in configs {
                        let instances = api::fetch_config_metadata(&config.name).await;
                        configs_list.push(PageConfig {
                            config: config,
                            instances: instances,
                        });
                    }
                }
                Err(err) => error!("Error fetching configs {}", err.to_string()),
            }
            configs_list
        },
    );

    view! { cx,
        <div>
            // Header
            <div style="display: flex; justify-content: end; gap: 10px">
                <a href="/login">{"Login"}</a>
                <a href="/admin">{"Admin"}</a>
                <a href="/add-config">{"Add Config"}</a>
                <a href="/add-label">{"Add Label"}</a>
            </div>

            {"Project "}
            <select>
                {move || match pd.read(cx) {
                    Some(data) => {
                        let projects = move || data.projects.clone();
                        view! { cx,
                            <For
                                each=projects
                                key=|p| p.uuid.clone()
                                view=move |cx, project: YakManProject| view! {cx,
                                    <option>{project.name}</option>
                                }
                            />
                        }.into_view(cx)
                    },
                    None => view! { cx, }.into_view(cx)
                }}
            </select>

            <div style="display: flex; flex-direction: column; align-items: center">
                <div>
                    <h1>{ "Configs" }</h1>

                    {move || match page_data.read(cx) {
                        None => view! { cx, <p>"Loading..."</p> }.into_view(cx),
                        Some(configs) => {
                            view! { cx,
                                {configs.into_iter().map(|config| view! {cx,
                                    <ConfigRow config={config} />
                                }).collect::<Vec<_>>()}
                            }.into_view(cx)
                        }
                    }}

                </div>
            </div>

        </div>
    }
}

#[component]
pub fn config_row(cx: Scope, #[prop()] config: PageConfig) -> impl IntoView {
    let create_config_instance_link = format!("/create-instance/{}", config.config.name);

    view! { cx,
        <div style="border: solid; border-radius: 6px; padding: 0px 20px; margin: 8px; min-width: 50vw">
            <div style="border-bottom: solid 2px; display: flex; justify-content: space-between; align-items: center">
                <h2>{&config.config.name}</h2>
                <a href={create_config_instance_link}>{"+"}</a> // TODO: use button instead
            </div>

            {config.instances.iter().map(|instance| {
                view! { cx,
                    <ConfigInstanceRow
                        instance={instance.clone()}
                    />
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn config_instance_row(cx: Scope, #[prop()] instance: ConfigInstance) -> impl IntoView {
    let labels_text = instance
        .labels
        .iter()
        .map(|label| format!("{}={}", label.label_type, label.value))
        .collect::<Vec<String>>()
        .join(", ");

    let config_name = &instance.config_name;
    let instance_id = &instance.instance;

    let view_link = format!("/api/configs/{config_name}/instances/{instance_id}/data");
    let edit_link = format!("/edit-instance/{config_name}/{instance_id}");
    let history_link = format!("/history/{config_name}/{instance_id}");
    let approval_link = format!("/apply/{config_name}/{instance_id}");

    view! { cx,
        <div
            key={instance.instance.clone()}
            style="display: flex; gap: 10px; justify-content: space-between"
        >
            <p>
                <a href={view_link} target="_blank">
                    { &instance.instance }
                </a>

            </p>
            <p>
                <a href={edit_link}>
                    { "Edit" }
                </a>
            </p>

            <p>
                <a href={history_link}>
                    { "History" }
                </a>
            </p>

            <div>
                {move || match &instance.pending_revision {
                    Some(_) => view! {cx,
                        <p>
                            <a href={&approval_link}>
                                { "Pending Change" }
                            </a>
                        </p>
                }.into_any(),
                    None => view! {cx,
                        <div />
                    }.into_any()
                }}

            </div>

            <p>{format!("{}", labels_text)}</p>
        </div>
    }
}
