use yak_man_core::model::{ConfigInstance, LabelType};
use yew::{
    function_component, html, use_effect_with_deps, use_state, Html, Properties, UseStateHandle,
};

use crate::{api, PageConfig, PageData};

#[function_component(ConfigListPage)]
pub fn config_list_page() -> Html {
    let page_data: UseStateHandle<Option<PageData>> = use_state(|| None);

    {
        let page_data = page_data.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let mut configs_list = vec![];

                    for config in api::fetch_configs().await {
                        let instances = api::fetch_instance_metadata(&config.name).await;
                        configs_list.push(PageConfig {
                            config: config,
                            instances: instances,
                        });
                    }

                    let labels: Vec<LabelType> = api::fetch_labels().await;

                    page_data.set(Some(PageData {
                        configs: configs_list,
                        labels: labels,
                    }));
                });
            },
            (),
        );
    }

    if page_data.is_none() {
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
                    {page_data.as_ref().unwrap().configs.iter().map(|config| {
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
