mod api;
mod components;
mod routes;

use components::{AddConfigPage, AddLabelPage, ConfigListPage};
use routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::{CreateConfigInstancePage, EditConfigInstancePage, RevisionHistoryPage};

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <ConfigListPage /> },
        Route::AddConfigPage => html! { <AddConfigPage /> },
        Route::AddLabelPage => html! { <AddLabelPage /> },
        Route::CreateConfigInstancePage { config_name } => html! {
            <CreateConfigInstancePage config_name={config_name} />
        },
        Route::EditConfigInstancePage {
            config_name,
            instance,
        } => html! {
            <EditConfigInstancePage
                config_name={config_name}
                instance={instance}
            />
        },
        Route::RevisionHistoryPage {
            config_name,
            instance,
        } => html! {
            <RevisionHistoryPage
                config_name={config_name}
                instance={instance}
            />
        },
        Route::NotFound => html! { <h1>{ "Not Found" }</h1> },
    }
}
