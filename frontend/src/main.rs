use gloo_net::http::Request;
use yew::prelude::*;
use yak_man_core::model::Config;
use serde::Deserialize;

// #[derive(Clone, PartialEq, Deserialize)]
// struct Video {
//     id: usize,
//     title: String,
//     speaker: String,
//     url: String,
// }

#[function_component(App)]
fn app() -> Html {
    let videos = use_state(|| vec![]);

    {
        let videos = videos.clone();
        use_effect_with_deps(
            move |_| {
                let videos = videos.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_videos: Vec<Config> =
                        Request::get("/api/configs")
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

    html! {
      <>
        <h1>{ "Hello World" }</h1>

        <p>{"Hello2"}</p>
      </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
