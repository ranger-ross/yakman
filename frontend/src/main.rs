use gloo_console::log;
use gloo_net::http::Request;
use yak_man_core::model::{Config, ConfigInstance, LabelType};
use yew::prelude::*;

#[derive(Debug, PartialEq)]
struct PageConfig {
    config: Config,
    instances: Vec<ConfigInstance>,
}

#[derive(Debug, PartialEq)]
struct PageData {
    configs: Vec<PageConfig>,
    labels: Vec<LabelType>,
}

#[function_component(App)]
fn app() -> Html {
    let page_data: UseStateHandle<Option<PageData>> = use_state(|| None);

    {
        let page_data = page_data.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let mut configs_list = vec![];

                    for config in fetch_configs().await {
                        let instances = fetch_instance_metadata(&config.name).await;
                        configs_list.push(PageConfig {
                            config: config,
                            instances: instances,
                        });
                    }

                    let labels: Vec<LabelType> = fetch_labels().await;

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
      <>
        <h1>{ "Configs" }</h1>

        {page_data.as_ref().unwrap().configs.iter().map(|config| {
            html! {
                <>
                    <h2>{&config.config.name}</h2>

                    {config.instances.iter().map(|instance| {
                        html! {
                            <ConfigInstanceRow key={instance.instance.clone()} instance={instance.clone()} />
                        }
                    }).collect::<Html>()}

                </>
            }
        }).collect::<Html>()}

      </>
    }
}

#[derive(Properties, PartialEq)]
struct ConfigInstanceRowProps {
    instance: ConfigInstance,
}

#[function_component(ConfigInstanceRow)]
fn config_instance_row(props: &ConfigInstanceRowProps) -> Html {
    let instance = &props.instance;
    html! {
        <p key={instance.instance.clone()}>{format!("{}: {}", instance.instance, "TODO: Add labels")}</p>
    }
}

async fn fetch_configs() -> Vec<Config> {
    return Request::get("/api/configs")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}

async fn fetch_labels() -> Vec<LabelType> {
    return Request::get("/api/labels")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}

async fn fetch_instance_metadata(name: &str) -> Vec<ConfigInstance> {
    return Request::get(&format!("/api/instances/{name}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}

fn main() {
    yew::Renderer::<App>::new().render();
}
