mod api;

use std::{collections::HashMap, rc::Rc, cell::RefCell};

use gloo_console::log;
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
    #[at("/add-label")]
    AddLabelPage,
    #[at("/create-instance/:config_name")]
    CreateConfigInstancePage { config_name: String },
    #[at("/edit-instance/:config_name/:instance")]
    EditConfigInstancePage {
        config_name: String,
        instance: String,
    },
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
        Route::EditConfigInstancePage {
            config_name,
            instance,
        } => {
            html! {
                <EditConfigInstancePage
                    config_name={config_name}
                    instance={instance}
                />
            }
        }
        Route::AddConfigPage => html! { <AddConfigPage /> },
        Route::AddLabelPage => html! { <AddLabelPage /> },
        Route::NotFound => html! { <h1>{ "Not Found" }</h1> },
    }
}
#[function_component(AddConfigPage)]
fn add_config_page() -> Html {
    let input_value_handle = use_state(String::default);

    let selected_labels_state: UseStateHandle<HashMap<String, Option<String>>> = use_state(HashMap::new);

    let input_value = (*input_value_handle).clone();

    let on_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        input_value_handle.set(value); // TODO: validate for duplicates?
    });

    let on_add_clicked = move |_| {
        let input_value = input_value.clone();
        wasm_bindgen_futures::spawn_local(async move {
            api::create_config(&input_value).await;
        });
    };

    let labels: UseStateHandle<Vec<LabelType>> = use_state(|| vec![]);

    {
        let label_data = labels.clone();
        let selected_labels_state = selected_labels_state.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let data = api::fetch_labels().await;

                    let mut m = HashMap::new();

                    for label in &data {
                        m.insert(String::from(&label.name), None);
                    }

                    selected_labels_state.set(m);

                    label_data.set(data);
                });
            },
            (),
        );
    }

    let selected_labels_state_value = (*selected_labels_state).clone();

    log!(
        "selected_labels_state_value = ",
        selected_labels_state_value.len()
    );

    let on_labels_changed: Callback<HashMap<String, Option<String>>> =
        Callback::from(|data: HashMap<String, Option<String>>| {
            log!("labels changed! len = ", data.len());

            

        });

    html! {
        <div>
            <h1>{"Add Config"}</h1>

            {"Name: "} <input onchange={on_change} />

            <LabelSelection
                labels={labels.to_vec()}
                selected_labels_state={selected_labels_state_value}
                on_change={on_labels_changed}
            />

            <br />
            <br />

            <button onclick={on_add_clicked}>{"Create"}</button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct LabelSelectionProps {
    labels: Vec<LabelType>,
    selected_labels_state: HashMap<String, Option<String>>,
    on_change: Callback<HashMap<String, Option<String>>>,
}

#[function_component(LabelSelection)]
fn label_selection(props: &LabelSelectionProps) -> Html {
    let selected_labels = Rc::new(RefCell::new(props.selected_labels_state.clone()));

    let on_change = Rc::new(props.on_change.clone());

    let on_select_change = Callback::from(move |event: Event| {
        let input = event.target_unchecked_into::<HtmlInputElement>(); // TODO: This sucks
        let value = input.value();
        let mut selected = selected_labels.borrow().clone();
        let old_value = selected.get_mut(&input.name()).unwrap();
        *old_value = Some(value);

        on_change.emit(selected);
    });

    html! {
        <>
            {props.labels.iter().map(|label| html! {
                <>
                    <br />
                    {&label.name}
                    <select onchange={&on_select_change} name={String::from(&label.name)}>
                        <option value="none">{"None"}</option>
                        {label.options.iter().map(|option| html! {
                            <option
                                value={option.clone()}
                                selected={
                                    let is_selected = props.selected_labels_state.get(&label.name).unwrap().clone().unwrap_or(String::from("")) == option.clone();
                                    log!("test", is_selected);
                                    is_selected
                                }
                            >{option}</option>
                        }).collect::<Html>()}
                    </select>
                </>
            }).collect::<Html>()}
        </>
    }
}

#[function_component(AddLabelPage)]
fn add_label_page() -> Html {
    let name = use_state(String::default);
    let name_value = (*name).clone();
    let prioity = use_state(String::default);
    let prioity_value = (*prioity).clone();
    let description = use_state(String::default);
    let description_value = (*description).clone();

    let on_name_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        name.set(value); // TODO: validate for duplicates?
    });

    let on_prioity_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        prioity.set(value); // TODO: validate for duplicates?
    });

    let on_description_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        description.set(value); // TODO: validate for duplicates?
    });

    let on_add_clicked = move |_| {
        let name = name_value.clone();
        let prioity = prioity_value.clone();
        let description = description_value.clone();
        wasm_bindgen_futures::spawn_local(async move {
            api::create_label(LabelType {
                name: name,
                description: description,
                priority: prioity.parse().unwrap(),
                options: vec![],
            })
            .await;
        });
    };

    html! {
        <div>
            <h1>{"Add Label"}</h1>
            <div>{"Name: "} <input onchange={on_name_change} /></div>
            <div>{"Prioity: "} <input onchange={on_prioity_change} /></div>
            <div>{"Description: "} <input onchange={on_description_change} /></div>

            // TODO: Support Options

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
struct EditConfigInstancePageProps {
    config_name: String,
    instance: String,
}

#[function_component(EditConfigInstancePage)]
fn edit_config_instance_page(props: &EditConfigInstancePageProps) -> Html {
    let input_value_handle = use_state(String::default);
    let input_value = (*input_value_handle).clone();

    let on_change = Callback::from(move |e: Event| {
        let value = e.target_unchecked_into::<HtmlTextAreaElement>().value();
        input_value_handle.set(value);
    });

    let config_name = props.config_name.clone();
    let instance = props.instance.clone();
    let on_add_clicked = move |_| {
        let config_name = config_name.clone(); // TODO: maybe handle this better?
        let instance = instance.clone();
        let input_value = input_value.clone();
        wasm_bindgen_futures::spawn_local(async move {
            api::update_config_instance(&config_name, &instance, &input_value).await;
        });
    };

    html! {
        <div>
            <h1>{format!("Edit Config Instance {} -> {}", props.config_name, props.instance)}</h1>

            <h3>{"Data"}</h3>
            <textarea onchange={on_change} />

            <br />

            <button onclick={Callback::from(on_add_clicked)}>{"Add"}</button>
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
            api::create_config_instance(&config_name, &input_value).await;
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

    let view_link = format!(
        "/api/config/{}/instance/{}",
        instance.config_name, instance.instance
    );

    let edit_link = format!(
        "/edit-instance/{}/{}",
        instance.config_name, instance.instance
    );
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

            <p>{format!("{}", labels_text)}</p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
