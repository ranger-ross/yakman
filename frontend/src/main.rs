use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
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
