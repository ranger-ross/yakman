mod api;
mod components;

use components::add_config_page::*;
use components::add_label_page::*;
use components::add_project_page::*;
use components::admin_page::*;
use components::apply_config_page::*;
use components::config_list_page::*;
use components::login_page::*;
use components::modify_config_instance::*;
use components::revision_history::*;
use leptos::*;
use leptos_router::*;

use crate::components::AdminPage;
use crate::components::LoginPage;

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, <AppRouter/> })
}

#[derive(Clone, Debug)]
struct GlobalState {
    is_login_needed: bool,
}

#[component]
pub fn header(cx: Scope) -> impl IntoView {
    view! { cx,
        <div style="display: flex; justify-content: end; gap: 10px; margin-bottom: 0.5rem">
            <a href="/">{"YakMan"}</a>
            <div style="flex-grow: 1" />
            <a href="/login">{"Login"}</a>
            <a href="/admin">{"Admin"}</a>
            <a href="/add-config">{"Add Config"}</a>
            <a href="/add-label">{"Add Label"}</a>
            <a href="/add-project">{"Add Project"}</a>
        </div>
    }
}

#[component]
pub fn AppRouter(cx: Scope) -> impl IntoView {
    let state = create_rw_signal(
        cx,
        GlobalState {
            is_login_needed: false,
        },
    );
    provide_context(cx, state);

    let attempt_to_refresh_token = move || {
        spawn_local(async move {
            match api::refresh_token().await {
                Ok(_) => {
                    let _ = window().location().reload();
                }
                Err(_) => state.update(|state| state.is_login_needed = true),
            };
        });
    };

    spawn_local(async move {
        match api::fetch_user_roles().await {
            Ok(role_data) => {
                if role_data.roles.len() == 0 && role_data.global_roles.len() == 0 {
                    attempt_to_refresh_token();
                }
            }
            Err(e) => match e {
                api::RequestError::UnexpectedHttpStatus(status) => {
                    if status >= 400 && status < 500 {
                        attempt_to_refresh_token();
                    }
                }
                e => error!("failed to fetch user roles {e:?}"),
            },
        }
    });

    let is_login_needed = move || state().is_login_needed;

    view! { cx,
        <Router>
            <Header />
            <main>
                <Routes>
                    <Route
                        path="/login"
                        view=move |cx| view! { cx, <LoginPage /> }
                    />
                    <Route
                        path="/oauth-callback"
                        view=move |cx| view! { cx, <OauthCallbackPage /> }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/"
                        view=move |cx| view! { cx, <ConfigListPage /> }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/add-project"
                        view=move |cx| view! { cx, <AddProjectPage /> }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/admin"
                        view=move |cx| view! { cx, <AdminPage /> }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/add-config"
                        view=move |cx| view! { cx, <AddConfigPage /> }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/add-label"
                        view=move |cx| view! { cx, <AddLabelPage /> }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/apply/:config_name/:instance"
                        view=move |cx| view! { cx, <ApplyConfigPage /> }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/create-instance/:config_name"
                        view=move |cx| view! { cx, <CreateConfigInstancePage /> }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/edit-instance/:config_name/:instance"
                        view=move |cx| view! { cx, <EditConfigInstancePage /> }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/history/:config_name/:instance"
                        view=move |cx| view! { cx, <RevisionHistoryPage /> }
                    />
                </Routes>
            </main>
        </Router>
    }
}
