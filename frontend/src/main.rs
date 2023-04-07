use gloo_net::http::Request;
use yak_man_core::model::{Config, LabelType, ConfigInstance};
use yew::prelude::*;
use gloo_console::log;

#[function_component(App)]
fn app() -> Html {
    let configs = use_state(|| vec![]);
    let labels = use_state(|| vec![]);
    let first_instances = use_state(|| vec![]);

    {
        let configs = configs.clone();
        let first_instances = first_instances.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_configs: Vec<Config> = fetch_configs().await;
                    
                    let first_config_name = &fetched_configs[0].name.clone();
                    configs.set(fetched_configs);

                    log!("Hello -> ", first_config_name);
                    let f = fetch_instance_metadata(&first_config_name).await;
                    first_instances.set(f);
                    

                    let fetched_labels: Vec<LabelType> = fetch_labels().await;
                    labels.set(fetched_labels);
                });
            },
            (),
        );
    }


    let first_instance_as_html: Html = first_instances.iter()
    .map(|instance| {
        html! {
            <p key={instance.instance.clone()}>{format!("{}: {}", instance.instance, "TODO: Add labels")}</p>
        }
    })
    .collect();

    let configs_as_html: Html = configs
        .iter()
        .map(|video| {
            html! {
                <p key={video.name.clone()}>{format!("{}: {}", video.name, video.description)}</p>
            }
        })
        .collect();

    html! {
      <>
        <h1>{ "Hello World" }</h1>

        {configs_as_html}

        <h1>{ "First Instance" }</h1>


        {first_instance_as_html}
      </>
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
