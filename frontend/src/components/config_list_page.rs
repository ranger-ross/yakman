use crate::api;
use chrono::{TimeZone, Utc};
use leptos::*;
use leptos_router::{use_navigate, use_query_map};
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
    let query = use_query_map(cx);
    let selected_project_uuid = move || query.with(|params| params.get("project").cloned());

    let update_navigation_url = move |project_uuid: &str| {
        let navigate = use_navigate(cx);
        navigate(&format!("?project={project_uuid}"), Default::default()).unwrap()
    };

    let pd = create_resource(
        cx,
        || (),
        move |_| async move {
            let projects = api::fetch_projects().await.unwrap();
            PageData { projects: projects }
        },
    );

    let selected_project = move || {
        return pd.read(cx).map(|data| {
            let first = &data.projects[0];
            data.projects
                .iter()
                .find(|p| p.uuid == selected_project_uuid().unwrap_or(String::from("not-found")))
                .unwrap_or(first)
                .clone()
        });
    };

    let page_data = create_resource(cx, selected_project, move |project| async move {
        let mut configs_list: Vec<PageConfig> = vec![];

        if let Some(project_uuid) = project.map(|p| p.uuid) {
            match api::fetch_configs(Some(project_uuid)).await {
                Ok(configs) => {
                    for config in configs {
                        let instances = api::fetch_config_metadata(&config.name).await;
                        configs_list.push(PageConfig {
                            config: config,
                            instances: instances,
                        });
                    }
                }
                Err(err) => {
                    error!("Error fetching configs {}", err.to_string());
                }
            }
        }

        configs_list
    });

    let on_project_change = move |ev| {
        let value = event_target_value(&ev);

        let (index, _) = pd
            .read(cx)
            .expect("Page data should be loaded before user can change projects")
            .projects
            .iter()
            .enumerate()
            .find(|(_, project)| project.uuid == value)
            .expect("The selected project should have been in the page data list");

        log!("Project Changed! {index:?}");
        update_navigation_url(&value);
    };

    view! { cx,
        <div>
            {"Project "}
            <select on:change=on_project_change>
                {move || match pd.read(cx) {
                    Some(data) => {
                        let projects = move || data.projects.clone();
                        view! { cx,
                            <For
                                each=projects
                                key=|p| p.uuid.clone()
                                view=move |cx, project: YakManProject| view! {cx,
                                    <option
                                        value=&project.uuid selected={project.uuid == selected_project_uuid().unwrap_or("other".to_string())}
                                    >
                                        {project.name}
                                    </option>
                                }
                            />
                        }.into_view(cx)
                    },
                    None => view! { cx, }.into_view(cx)
                }}
            </select>


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
    }
}

#[component]
pub fn config_row(cx: Scope, #[prop()] config: PageConfig) -> impl IntoView {
    let create_config_instance_link = format!("/create-instance/{}", config.config.name);
    let has_at_least_one_instance = config.instances.len() > 0;
    view! { cx,
        <div class="bg-white border-2 border-gray-200 m-2 p-4">
            <div class="flex justify-between">
                <h3 class="text-gray-900 font-bold text-lg">{move || config.config.name.clone()}</h3>
                <LinkWithChrevon href={create_config_instance_link}>"Add Instance"</LinkWithChrevon>
            </div>

            <Show
                when=move || has_at_least_one_instance
                fallback=move |_| view! {cx,
                    <EmptyConfigRow />
                }
            >
                {config.instances.iter().map(|instance| {
                    view! { cx,
                        <ConfigInstanceRow
                            instance={instance.clone()}
                        />
                    }
                }).collect::<Vec<_>>()}
            </Show>
        </div>
    }
}

#[component]
pub fn empty_config_row(cx: Scope) -> impl IntoView {
    view! { cx,
        <>
            <div class="shadow-sm w-full h-1 mb-3"/>
            <div class="flex justify-center">
                <span class="text-gray-700">"No config instances"</span>
            </div>
        </>
    }
}

#[component]
pub fn config_instance_row(cx: Scope, #[prop()] instance: ConfigInstance) -> impl IntoView {
    let config_name = &instance.config_name;
    let instance_id = &instance.instance;

    let last_updated = get_last_updated_timestamp(&instance).map(|last_updated| {
        let datetime = Utc.timestamp_millis_opt(last_updated).unwrap();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    });

    let view_link = format!("/api/v1/configs/{config_name}/instances/{instance_id}/data");
    let edit_link = format!("/edit-instance/{config_name}/{instance_id}");
    let history_link = format!("/history/{config_name}/{instance_id}");
    let approval_link = format!("/apply/{config_name}/{instance_id}");

    view! { cx,
        <>
            <div class="shadow-sm w-full h-1 mb-3"/>

            <div class="flex justify-between">
                <div>
                    <a href={view_link} target="_blank" class="font-bold">{instance_id}</a>
                    <div class="text-gray-500">"Last Updated: "{last_updated}</div>
                </div>

                <div class="flex flex-col items-end">
                    <LinkWithChrevon href={edit_link}>"Edit"</LinkWithChrevon>
                    <LinkWithChrevon href={history_link}>"History"</LinkWithChrevon>

                    <Show
                        when=move || instance.pending_revision.is_some()
                        fallback=|_| view! { cx, }
                    >
                        <LinkWithChrevon href={approval_link.clone()}>"Review Changes"</LinkWithChrevon>
                    </Show>

                </div>
            </div>
        </>
    }
}

fn get_last_updated_timestamp(instance: &ConfigInstance) -> Option<i64> {
    return instance.changelog.iter().last().map(|c| c.timestamp_ms);
}

#[component]
pub fn link_with_chrevon(cx: Scope, #[prop()] href: String, children: Children) -> impl IntoView {
    view! { cx,
        <a
            class="text-indigo-600 flex items-center text-lg"
            href={href}
        >
            {children(cx)}
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
            </svg>
        </a>
    }
}
