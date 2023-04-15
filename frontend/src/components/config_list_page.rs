use gloo_console::error;
use yak_man_core::model::{Config, ConfigInstance, LabelType};
use yew::{
    function_component, hook, html, use_effect_with_deps, use_state, Html, Properties,
    UseStateHandle,
};

use crate::api;

#[derive(Debug, PartialEq, Clone)]
struct PageConfig {
    config: Config,
    instances: Vec<ConfigInstance>,
}

#[derive(Debug, PartialEq, Clone)]
struct PageData {
    configs: Vec<PageConfig>,
    labels: Vec<LabelType>,
    is_loading: bool,
}

#[hook]
fn use_config_data() -> UseStateHandle<PageData> {
    let page_data: UseStateHandle<PageData> = use_state(|| PageData {
        configs: vec![],
        labels: vec![],
        is_loading: true,
    });

    {
        let page_data = page_data.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let mut configs_list = vec![];

                    match api::fetch_configs().await {
                        Ok(configs) => {
                            for config in configs {
                                let instances = api::fetch_instance_metadata(&config.name).await;
                                configs_list.push(PageConfig {
                                    config: config,
                                    instances: instances,
                                });
                            }
                        }
                        Err(err) => error!("Error fetching configs", err.to_string()),
                    }

                    match api::fetch_labels().await {
                        Ok(labels) => {
                            page_data.set(PageData {
                                configs: configs_list,
                                labels: labels,
                                is_loading: false,
                            });
                        }
                        Err(err) => error!("Error loading data", err.to_string()),
                    }
                });
            },
            (),
        );
    }
    return page_data;
}

#[function_component(ConfigListPage)]
pub fn config_list_page() -> Html {
    let page_data: UseStateHandle<PageData> = use_config_data();

    if page_data.is_loading {
        return html! {
            <div> {"Loading..."} </div>
        };
    }

    html! {
        <div>
            // Header
            <div style="display: flex; justify-content: end; gap: 10px">
                <a href="/add-config">{"Add Config"}</a>

                <a href="/add-label">{"Add Label"}</a>
            </div>

            <div style="display: flex; flex-direction: column; align-items: center">
                <div>
                    <h1>{ "Configs" }</h1>
                    {page_data.configs.iter().map(|config| {
                        html! { <ConfigRow config={config.clone()} /> }
                    }).collect::<Html>()}
                </div>
            </div>

        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ConfigRowProps {
    config: PageConfig,
}

#[function_component(ConfigRow)]
fn config_row(props: &ConfigRowProps) -> Html {
    let create_config_instance_link = format!("/create-instance/{}", props.config.config.name);
    html! {
        <div style="border: solid; border-radius: 6px; padding: 0px 20px; margin: 8px; min-width: 50vw">
            <div style="border-bottom: solid 2px; display: flex; justify-content: space-between; align-items: center">
                <h2>{&props.config.config.name}</h2>
                <a href={create_config_instance_link}>{"+"}</a> // TODO: use button instead
            </div>


            {props.config.instances.iter().map(|instance| {
                html! {
                    <ConfigInstanceRow
                        key={instance.instance.clone()}
                        instance={instance.clone()}
                    />
                }
            }).collect::<Html>()}
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ConfigInstanceRowProps {
    instance: ConfigInstance,
}

#[function_component(ConfigInstanceRow)]
fn config_instance_row(props: &ConfigInstanceRowProps) -> Html {
    let instance = &props.instance;
    let labels_text = instance
        .labels
        .iter()
        .map(|label| format!("{}={}", label.label_type, label.value))
        .collect::<Vec<String>>()
        .join(", ");

    let config_name = &instance.config_name;
    let instance_id = &instance.instance;

    let view_link = format!("/api/config/{config_name}/instance/{instance_id}");
    let edit_link = format!("/edit-instance/{config_name}/{instance_id}");
    let history_link = format!("/history/{config_name}/{instance_id}");

    html! {
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


            <p>{format!("{}", labels_text)}</p>
        </div>
    }
}
