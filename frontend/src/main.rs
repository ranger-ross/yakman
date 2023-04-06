use gloo_net::http::Request;
use yak_man_core::model::Config;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let configs = use_state(|| vec![]);

    {
        let configs = configs.clone();
        use_effect_with_deps(
            move |_| {
                let videos = configs.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_videos: Vec<Config> = Request::get("/api/configs")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    videos.set(fetched_videos);
                });
            },
            (),
        );
    }

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

        <p>{"Hello2"}</p>

        {configs_as_html}
      </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
