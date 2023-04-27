use gloo_net::http::Request;
use leptos::*;
use std::fmt;
use std::{collections::HashMap, error::Error};
use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, LabelType};

fn main() {
    mount_to_body(|cx| {
        view! { cx,
            <div>
                <AddLabelPage />
            </div>
        }
    })
}

#[derive(Debug)]
pub enum RequestError {
    Reqwest(gloo_net::Error),
    Json(serde_json::Error),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RequestError::Reqwest(ref e) => e.fmt(f),
            RequestError::Json(ref e) => e.fmt(f),
        }
    }
}

impl Error for RequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            RequestError::Reqwest(ref e) => Some(e),
            RequestError::Json(ref e) => Some(e),
        }
    }
}

impl From<gloo_net::Error> for RequestError {
    fn from(err: gloo_net::Error) -> RequestError {
        RequestError::Reqwest(err)
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(err: serde_json::Error) -> RequestError {
        RequestError::Json(err)
    }
}

pub async fn create_label(label: LabelType) -> Result<(), RequestError> {
    let body = serde_json::to_string(&label)?;
    Request::put("/api/labels")
        .body(body)
        .header("content-type", "application/json")
        .send()
        .await?;
    return Ok(());
}

#[component]
pub fn add_label_page(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::from(""));
    let (prioity, set_prioity) = create_signal(cx, String::from(""));
    let (description, set_description) = create_signal(cx, String::from(""));
    let (options, set_options) = create_signal(cx, String::from(""));

    let add_todo = create_action(cx, |d: &(String, String, String, String)| {
        let (name, description, prioity, options) = d;

        let options = options
            .split(",")
            .into_iter()
            .map(String::from)
            .filter(|o| !o.is_empty())
            .collect::<Vec<String>>();

        let label = LabelType {
            name: name.clone(),
            description: description.clone(),
            priority: prioity.parse().unwrap(),
            options: options,
        };

        async move { 
            log!("from action! {:?}", label);
            create_label(label).await.unwrap()
        }
    });

    // let on_add_clicked = move |_| {
    //     let options = options()
    //         .split(",")
    //         .into_iter()
    //         .map(String::from)
    //         .filter(|o| !o.is_empty())
    //         .collect::<Vec<String>>();

    //     log!("Options: {options:?}");
    //     // wasm_bindgen_futures::spawn_local(async move {
    //     //     match api::create_label(LabelType {
    //     //         name: name,
    //     //         description: description,
    //     //         priority: prioity.parse().unwrap(),
    //     //         options: options,
    //     //     })
    //     //     .await
    //     //     {
    //     //         Ok(()) => {}
    //     //         Err(err) => error!("Error creating label", err.to_string()),
    //     //     };
    //     //     navigator.push(&Route::Home);
    //     // });
    // };

    view! { cx,
        <div>
            <h1>{"Add Label"}</h1>
            <div>{"Name: "} <input type="text" on:input=move |ev| set_name(event_target_value(&ev)) prop:value=name /></div>
            <div>{"Prioity: "} <input  type="text" on:input=move |ev| set_prioity(event_target_value(&ev)) prop:value=prioity/></div>
            <div>{"Description: "} <input  type="text" on:input=move |ev| set_description(event_target_value(&ev)) prop:value=description /></div>
            <div>{"Options: "} <input  type="text" on:input=move |ev| set_options(event_target_value(&ev)) prop:value=options /></div>

            <br />



            <button on:click=move |_| add_todo.dispatch((name(), description(), prioity(), options()))>"Create"</button>

            <Show
                when=move || add_todo.pending().get()
                fallback=|cx| view! { cx, <div/> }
            >
            <div>"Loading" </div>
            </Show>


        </div>
    }
}

// #[function_component(AddLabelPage)]
// pub fn add_label_page() -> Html {
//     let navigator = use_navigator().unwrap();
//     let name = use_state(String::default);
//     let name_value = (*name).clone();
//     let prioity = use_state(String::default);
//     let prioity_value = (*prioity).clone();
//     let description = use_state(String::default);
//     let description_value = (*description).clone();
//     let options = use_state(String::default);
//     let options_value = (*options).clone();

//     let on_name_change = Callback::from(move |e: Event| {
//         // TODO: make sure input matches config name requirements
//         let value = e.target_unchecked_into::<HtmlInputElement>().value();
//         name.set(value); // TODO: validate for duplicates?
//     });

//     let on_prioity_change = Callback::from(move |e: Event| {
//         // TODO: make sure input matches config name requirements
//         let value = e.target_unchecked_into::<HtmlInputElement>().value();
//         prioity.set(value); // TODO: validate
//     });

//     let on_description_change = Callback::from(move |e: Event| {
//         // TODO: make sure input matches config name requirements
//         let value = e.target_unchecked_into::<HtmlInputElement>().value();
//         description.set(value);
//     });

//     let on_options_change = Callback::from(move |e: Event| {
//         // TODO: make sure input matches config name requirements
//         let value = e.target_unchecked_into::<HtmlInputElement>().value();
//         options.set(value); // TODO: validate
//     });

//     let on_add_clicked = move |_| {
//         let name = name_value.clone();
//         let prioity = prioity_value.clone();
//         let description = description_value.clone();
//         let navigator = navigator.clone();
//         let options = options_value
//             .split(",")
//             .into_iter()
//             .map(String::from)
//             .filter(|o| !o.is_empty())
//             .collect::<Vec<String>>();
//         wasm_bindgen_futures::spawn_local(async move {
//             match api::create_label(LabelType {
//                 name: name,
//                 description: description,
//                 priority: prioity.parse().unwrap(),
//                 options: options,
//             })
//             .await
//             {
//                 Ok(()) => {}
//                 Err(err) => error!("Error creating label", err.to_string()),
//             };
//             navigator.push(&Route::Home);
//         });
//     };

//     html! {
//         <div>
//             <h1>{"Add Label"}</h1>
//             <div>{"Name: "} <input onchange={on_name_change} /></div>
//             <div>{"Prioity: "} <input onchange={on_prioity_change} /></div>
//             <div>{"Description: "} <input onchange={on_description_change} /></div>
//             <div>{"Options: "} <input onchange={on_options_change} /></div>

//             <br />

//             <button onclick={on_add_clicked}>{"Create"}</button>
//         </div>
//     }
// }
