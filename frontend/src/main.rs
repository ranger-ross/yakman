use gloo_console::log;
use gloo_net::http::Request;
use yak_man_core::model::{Config, ConfigInstance, LabelType};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/create-instance/:config_name")]
    CreateConfigInstancePage { config_name: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Debug, PartialEq, Clone)]
struct PageConfig {
    config: Config,
    instances: Vec<ConfigInstance>,
}

#[derive(Debug, PartialEq)]
struct PageData {
    configs: Vec<PageConfig>,
    labels: Vec<LabelType>,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <MainView /> },
        Route::CreateConfigInstancePage { config_name } => {
            html! {
                <CreateConfigInstancePage config_name={config_name} />
            }
        }
        Route::NotFound => html! { <h1>{ "Not Found" }</h1> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[function_component(MainView)]
fn main_view() -> Html {
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
        <div style="display: flex; flex-direction: column; align-items: center">
            <div>
                <h1>{ "Configs" }</h1>
                {page_data.as_ref().unwrap().configs.iter().map(|config| {
                    html! { <ConfigRow config={config.clone()} /> }
                }).collect::<Html>()}
            </div>

        </div>
    }
}

#[derive(Properties, PartialEq)]
struct CreateConfigInstancePageProps {
    config_name: String,
}

#[function_component(CreateConfigInstancePage)]
fn create_config_instance_page(props: &CreateConfigInstancePageProps) -> Html {
    html! {
        <div>
            {&props.config_name}
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ConfigRowProps {
    config: PageConfig,
}

#[function_component(ConfigRow)]
fn config_row(props: &ConfigRowProps) -> Html {
    html! {
        <div style="border: solid; border-radius: 6px; padding: 0px 20px; margin: 8px; min-width: 50vw">
            <div style="border-bottom: solid 2px; display: flex; justify-content: space-between; align-items: center">
                <h2>{&props.config.config.name}</h2>
                <a href="/create-instance/testing-1">{"+"}</a> // TODO: use button instead
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

    let mut link = format!("/api/data/{}", instance.config_name);
    if instance.labels.len() > 0 {
        let labels_params = instance
            .labels
            .iter()
            .map(|label| format!("{}={}", label.label_type, label.value))
            .collect::<Vec<String>>()
            .join("&");
        link.push('?');
        link.push_str(&labels_params);
    }
    html! {
        <div
            key={instance.instance.clone()}
            style="display: flex; gap: 10px; justify-content: space-between"
        >
            <p>
                <a
                    target="_blank"
                    href={link}
                >
                    { &instance.instance }
                </a>
            </p>

            <p>{format!("{}", labels_text)}</p>
        </div>
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
