mod api;
mod components;
mod routes;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gloo_console::log;
use routes::Route;
use web_sys::HtmlInputElement;
use yak_man_core::model::{Config, ConfigInstance, LabelType};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::{CreateConfigInstancePage, EditConfigInstancePage, RevisionHistoryPage};

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
        Route::RevisionHistoryPage {
            config_name,
            instance,
        } => {
            html! {
                <RevisionHistoryPage
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
    let navigator = use_navigator().unwrap();
    let input_value_handle = use_state(String::default);
    let input_value = (*input_value_handle).clone();

    let on_change = Callback::from(move |e: Event| {
        // TODO: make sure input matches config name requirements
        let value = e.target_unchecked_into::<HtmlInputElement>().value();
        input_value_handle.set(value); // TODO: validate for duplicates?
    });

    let on_add_clicked = move |_| {
        let navigator = navigator.clone();
        let input_value = input_value.clone();
        wasm_bindgen_futures::spawn_local(async move {
            api::create_config(&input_value).await;
            navigator.push(&Route::Home);
        });
    };

    let labels: UseStateHandle<Vec<LabelType>> = use_state(|| vec![]);

    {
        let label_data = labels.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let data = api::fetch_labels().await;
                    label_data.set(data);
                });
            },
            (),
        );
    }

    html! {
        <div>
            <h1>{"Add Config"}</h1>

            {"Name: "} <input onchange={on_change} />

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
                        <option value="none" selected={true}>{"None"}</option>
                        {label.options.iter().map(|option| html! {
                            <option
                                value={option.clone()}
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
    let navigator = use_navigator().unwrap();
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
        let navigator = navigator.clone();
        wasm_bindgen_futures::spawn_local(async move {
            api::create_label(LabelType {
                name: name,
                description: description,
                priority: prioity.parse().unwrap(),
                options: vec![],
            })
            .await;
            navigator.push(&Route::Home);
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

fn main() {
    yew::Renderer::<App>::new().render();
}
