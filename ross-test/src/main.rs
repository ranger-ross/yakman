mod api;
mod components;

use components::add_config_page::*;
use components::modify_config_instance::*;
use components::add_label_page::*;
use components::apply_config_page::*;
use components::config_list_page::*;
use components::revision_history::*;
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
                        path="/add-config"
                        view=move |cx| view! { cx,  <AddConfigPage /> }
                    />
                    <Route
                        path="/add-label"
                        view=move |cx| view! { cx,  <AddLabelPage /> }
                    />
                    <Route
                        path="/apply/:config_name/:instance"
                        view=move |cx| view! { cx,  <ApplyConfigPage /> }
                    />
                    <Route
                        path="/create-instance/:config_name"
                        view=move |cx| view! { cx,  <CreateConfigInstancePage /> }
                    />
                    <Route
                        path="/edit-instance/:config_name/:instance"
                        view=move |cx| view! { cx,  <EditConfigInstancePage /> }
                    />
                    <Route
                        path="/history/:config_name/:instance"
                        view=move |cx| view! { cx,  <RevisionHistoryPage /> }
                    />
                </Routes>
            </main>
        </Router>
    }
}
