use gloo_console::log;
use gloo_net::http::Request;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yak_man_core::model::{Config, ConfigInstance, LabelType};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/add-config")]
    AddConfigPage,
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
        Route::AddConfigPage => html! { <AddConfigPage /> },
        Route::NotFound => html! { <h1>{ "Not Found" }</h1> },
    }
}

#[function_component(AddConfigPage)]
fn add_config_page() -> Html {
    let input_value_handle = use_state(String::default);
    let input_value = (*input_value_handle).clone();

    let on_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        input_value_handle.set(value); // TODO: validate for duplicates?
    });

    let on_add_clicked = move |_| {
        let input_value = input_value.clone();
        wasm_bindgen_futures::spawn_local(async move {
            create_config(&input_value).await;
        });
    };

    html! {
        <div>
            <h1>{"Add Config"}</h1>

            {"Name: "} <input onchange={on_change} />

            <br />

            <button onclick={on_add_clicked}>{"Create"}</button>
        </div>
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
        <div>
            // Header
            <div style="display: flex; justify-content: end">
                <a href="/add-config">{"+"}</a>
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
struct CreateConfigInstancePageProps {
    config_name: String,
}

#[function_component(CreateConfigInstancePage)]
fn create_config_instance_page(props: &CreateConfigInstancePageProps) -> Html {
    let input_value_handle = use_state(String::default);
    let input_value = (*input_value_handle).clone();

    let on_change = Callback::from(move |e: Event| {
        let value = e.target_unchecked_into::<HtmlTextAreaElement>().value();
        input_value_handle.set(value);
    });

    let config_name = props.config_name.clone();
    let on_add_clicked = move |_| {
        let config_name = config_name.clone(); // TODO: maybe handle this better?
        let input_value = input_value.clone();
        wasm_bindgen_futures::spawn_local(async move {
            create_config_instance(&config_name, &input_value).await;
        });
    };

    html! {
        <div>
            <h1>{format!("Create Config Instance {}", props.config_name)}</h1>

            <h3>{"Data"}</h3>
            <textarea onchange={on_change} />

            <br />

            <button onclick={Callback::from(on_add_clicked)}>{"Add"}</button>
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

    let link = format!(
        "/api/config/{}/instance/{}",
        instance.config_name, instance.instance
    );
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

async fn fetch_instance_metadata(config_name: &str) -> Vec<ConfigInstance> {
    return Request::get(&format!("/api/instances/{config_name}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}

async fn create_config_instance(config_name: &str, data: &str) -> Vec<LabelType> {
    return Request::put(&format!("/api/config/{config_name}/data"))
        .body(data)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}

async fn create_config(config_name: &str) -> Vec<LabelType> {
    return Request::put(&format!("/api/config/{config_name}"))
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
