mod add_config_page;
mod add_label_page;
mod api;
mod config_list_page;

use add_config_page::*;
use add_label_page::*;
use config_list_page::*;
use leptos::*;
use leptos_router::*;

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, <RouterExample/> })
}

#[component]
pub fn RouterExample(cx: Scope) -> impl IntoView {
    view! { cx,
        <Router>
            <main>
                <Routes>
                    <Route
                        path="/"
                        view=move |cx| view! { cx,  <ConfigListPage /> }
                    />
                    <Route
                        path="add-config"
                        view=move |cx| view! { cx,  <AddConfigPage /> }
                    />
                    <Route
                        path="add-label"
                        view=move |cx| view! { cx,  <AddLabelPage /> }
                    />
                </Routes>
            </main>
        </Router>
    }
}

