use std::collections::HashMap;

use leptos::*;
use leptos_router::*;
use yak_man_core::model::LabelType;

use crate::api;

#[component]
pub fn create_config_instance_page(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    // TODO: use a better way to extract params
    let config_name = move || params.with(|params| params.get("config_name").cloned().unwrap());

    let (input, set_input) = create_signal(cx, String::from(""));
    let (selected_labels, set_selected_labels) =
        create_signal::<HashMap<String, Option<String>>>(cx, HashMap::new());

    let on_create = create_action(cx, move |_: &()| async move {
        let config_name = config_name();
        let input_value = input();
        let selected_labels = selected_labels();

        log!("Selected labels len = {}", selected_labels.len());

        let selected_labels: HashMap<String, String> = selected_labels
            .into_iter()
            .filter_map(|(key, v)| v.map(|value| (key, value)))
            .collect();

        match api::create_config_instance(&config_name, &input_value, selected_labels).await {
            Ok(()) => {
                let navigate = use_navigate(cx);
                _ = navigate("/", Default::default()); // TODO: Fix warnings (Question in Discord: https://discordapp.com/channels/1031524867910148188/1049869221636620300/1101740642478084156)
            }
            Err(err) => error!("Error creating config instance {}", err.to_string()),
        };
    });

    let page_data = create_resource(
        cx,
        || (),
        |_| async move {
            log!("fetching");
            let mut label_data = vec![];

            match api::fetch_labels().await {
                Ok(data) => label_data = data,
                Err(err) => error!("Error loading label: {}", err.to_string()),
            }

            label_data
        },
    );

    create_effect(cx, move |_| {
        if let Some(labels) = page_data.read(cx) {
            let mut m: HashMap<String, Option<String>> = HashMap::new();
            for label in &labels {
                m.insert(String::from(&label.name), None);
            }
            set_selected_labels(m);
        }
    });

    let labels = move || page_data.read(cx).unwrap_or(vec![]);

    view! { cx,
        <div>
            <h1>"Create Config Instance "{config_name}</h1>

            <h3>{"Data"}</h3>
            <textarea on:input=move |ev| set_input(event_target_value(&ev)) prop:value=input />

            <LabelSelection
                labels={Signal::derive(cx, labels)}
                selected_labels={selected_labels.into()}
                set_selected_labels={set_selected_labels}
             />

            <br />

            <button on:click=move |_| on_create.dispatch(())>{"Add"}</button>
        </div>
    }
}

// #[derive(Properties, PartialEq)]
// pub struct CreateConfigInstancePageProps {
//     pub config_name: String,
// }

// #[function_component(CreateConfigInstancePage)]
// pub fn create_config_instance_page(props: &CreateConfigInstancePageProps) -> Html {
//     let navigator = use_navigator().unwrap();
//     let input_value_handle = use_state(String::default);
//     let input_value = (*input_value_handle).clone();

//     let on_change = Callback::from(move |e: Event| {
//         let value = e.target_unchecked_into::<HtmlTextAreaElement>().value();
//         input_value_handle.set(value);
//     });

//     let config_name = props.config_name.clone();

//     let labels: UseStateHandle<Vec<LabelType>> = use_state(|| vec![]);
//     let selected_labels_state: UseStateHandle<HashMap<String, Option<String>>> =
//         use_state(HashMap::new);
//     let selected_labels_state_value = (*selected_labels_state).clone();
//     {
//         let label_data = labels.clone();
//         let selected_labels_state = selected_labels_state.clone();
//         use_effect_with_deps(
//             move |_| {
//                 wasm_bindgen_futures::spawn_local(async move {
//                     match api::fetch_labels().await {
//                         Ok(data) => {
//                             let mut m = HashMap::new();
//                             for label in &data {
//                                 m.insert(String::from(&label.name), None);
//                             }
//                             selected_labels_state.set(m);
//                             label_data.set(data);
//                         }
//                         Err(err) => error!("Error loading label", err.to_string()),
//                     }
//                 });
//             },
//             (),
//         );
//     }

//     let selected_labels_state_value_clone = (*selected_labels_state).clone();

//     let on_add_clicked = move |_| {
//         let config_name = config_name.clone(); // TODO: maybe handle this better?
//         let input_value = input_value.clone();
//         let selected_labels = selected_labels_state_value_clone.clone();
//         let navigator = navigator.clone();

//         log!("Selected labels len = ", selected_labels.len());

//         wasm_bindgen_futures::spawn_local(async move {
//             let selected_labels: HashMap<String, String> = selected_labels
//                 .into_iter()
//                 .filter_map(|(key, v)| v.map(|value| (key, value)))
//                 .collect();

//             match api::create_config_instance(&config_name, &input_value, selected_labels).await {
//                 Ok(()) => navigator.push(&Route::Home),
//                 Err(err) => error!("Error creating config instance", err.to_string()),
//             };
//         });
//     };

//     log!(
//         "selected_labels_state_value = ",
//         selected_labels_state_value.len()
//     );

//     let on_labels_changed: Callback<HashMap<String, Option<String>>> =
//         Callback::from(move |data: HashMap<String, Option<String>>| {
//             log!("labels changed! len = ", data.len());
//             selected_labels_state.set(data);
//         });

//     html! {
//         <div>
//             <h1>{format!("Create Config Instance {}", props.config_name)}</h1>

//             <h3>{"Data"}</h3>
//             <textarea onchange={on_change} />

//             <LabelSelection
//                 labels={labels.to_vec()}
//                 selected_labels_state={selected_labels_state_value}
//                 on_change={on_labels_changed}
//             />

//             <br />

//             <button onclick={Callback::from(on_add_clicked)}>{"Add"}</button>
//         </div>
//     }
// }

#[component]
pub fn label_selection(
    cx: Scope,
    #[prop()] labels: Signal<Vec<LabelType>>,
    #[prop()] selected_labels: Signal<HashMap<String, Option<String>>>,
    #[prop()] set_selected_labels: WriteSignal<HashMap<String, Option<String>>>,
) -> impl IntoView {
    let on_select_change = move |ev| {
        let el: web_sys::HtmlSelectElement = event_target(&ev);
        let name = el.name();
        let value = event_target_value(&ev);

        let mut selected = selected_labels();
        let old_value = selected.get_mut(&name).unwrap();
        *old_value = Some(value);

        set_selected_labels(selected);
    };

    view! { cx,
        <>
        {move || labels().iter().map(|label| view! { cx,
            <>
                <br />
                {&label.name}
                <select name={String::from(&label.name)} on:change=on_select_change>
                    <option value="none" selected={true}>{"None"}</option>
                    {label.options.iter().map(|option| view! { cx,
                        <option value={option}>
                            {option}
                        </option>
                    }).collect::<Vec<_>>()}
                </select>
            </>
        }).collect::<Vec<_>>()}
    </>
    }
}

// #[derive(Properties, PartialEq)]
// pub struct LabelSelectionProps {
//     labels: Vec<LabelType>,
//     selected_labels_state: HashMap<String, Option<String>>,
//     on_change: Callback<HashMap<String, Option<String>>>,
// }

// #[function_component(LabelSelection)]
// pub fn label_selection(props: &LabelSelectionProps) -> Html {
//     let selected_labels = Rc::new(RefCell::new(props.selected_labels_state.clone()));

//     let on_change = Rc::new(props.on_change.clone());

//     let on_select_change = Callback::from(move |event: Event| {
//         let input = event.target_unchecked_into::<HtmlInputElement>(); // TODO: This sucks
//         let value = input.value();
//         let mut selected = selected_labels.borrow().clone();
//         let old_value = selected.get_mut(&input.name()).unwrap();
//         *old_value = Some(value);

//         on_change.emit(selected);
//     });

//     html! {
//         <>
//             {props.labels.iter().map(|label| html! {
//                 <>
//                     <br />
//                     {&label.name}
//                     <select onchange={&on_select_change} name={String::from(&label.name)}>
//                         <option value="none" selected={true}>{"None"}</option>
//                         {label.options.iter().map(|option| html! {
//                             <option
//                                 value={option.clone()}
//                             >{option}</option>
//                         }).collect::<Html>()}
//                     </select>
//                 </>
//             }).collect::<Html>()}
//         </>
//     }
// }

// #[derive(Properties, PartialEq)]
// pub struct EditConfigInstancePageProps {
//     pub config_name: String,
//     pub instance: String,
// }

// #[function_component(EditConfigInstancePage)]
// pub fn edit_config_instance_page(props: &EditConfigInstancePageProps) -> Html {
//     let navigator = use_navigator().unwrap();
//     let input_value_handle = use_state(String::default);
//     let input_value = (*input_value_handle).clone();

//     let on_change = Callback::from(move |e: Event| {
//         let value = e.target_unchecked_into::<HtmlTextAreaElement>().value();
//         input_value_handle.set(value);
//     });

//     let labels: UseStateHandle<Vec<LabelType>> = use_state(|| vec![]);
//     let selected_labels_state: UseStateHandle<HashMap<String, Option<String>>> =
//         use_state(HashMap::new);
//     let selected_labels_state_value = (*selected_labels_state).clone();
//     {
//         let label_data = labels.clone();
//         let selected_labels_state = selected_labels_state.clone();
//         use_effect_with_deps(
//             move |_| {
//                 wasm_bindgen_futures::spawn_local(async move {
//                     match api::fetch_labels().await {
//                         Ok(data) => {
//                             let mut m = HashMap::new();
//                             for label in &data {
//                                 m.insert(String::from(&label.name), None);
//                             }
//                             selected_labels_state.set(m);
//                             label_data.set(data);
//                         }
//                         Err(err) => error!("Error loading label", err.to_string()),
//                     }
//                 });
//             },
//             (),
//         );
//     }

//     let config_name = props.config_name.clone();
//     let instance = props.instance.clone();
//     let selected_labels_state_value_clone = (*selected_labels_state).clone();
//     let on_add_clicked = move |_| {
//         let config_name = config_name.clone(); // TODO: maybe handle this better?
//         let instance = instance.clone();
//         let input_value = input_value.clone();
//         let selected_labels = selected_labels_state_value_clone.clone();
//         let navigator = navigator.clone();

//         log!("Selected labels len = ", selected_labels.len());

//         wasm_bindgen_futures::spawn_local(async move {
//             let selected_labels: HashMap<String, String> = selected_labels
//                 .into_iter()
//                 .filter_map(|(key, v)| v.map(|value| (key, value)))
//                 .collect();

//             match api::update_config_instance(
//                 &config_name,
//                 &instance,
//                 &input_value,
//                 selected_labels,
//             )
//             .await
//             {
//                 Ok(()) => navigator.push(&Route::Home),
//                 Err(err) => error!("Error updating config instance", err.to_string()),
//             };
//         });
//     };

//     log!(
//         "selected_labels_state_value = ",
//         selected_labels_state_value.len()
//     );

//     let on_labels_changed: Callback<HashMap<String, Option<String>>> =
//         Callback::from(move |data: HashMap<String, Option<String>>| {
//             log!("labels changed! len = ", data.len());
//             selected_labels_state.set(data);
//         });

//     html! {
//         <div>
//             <h1>{format!("Edit Config Instance {} -> {}", props.config_name, props.instance)}</h1>

//             <h3>{"Data"}</h3>
//             <textarea onchange={on_change} />

//             <LabelSelection
//                 labels={labels.to_vec()}
//                 selected_labels_state={selected_labels_state_value}
//                 on_change={on_labels_changed}
//             />

//             <br />

//             <button onclick={Callback::from(on_add_clicked)}>{"Update"}</button>
//         </div>
//     }
// }
