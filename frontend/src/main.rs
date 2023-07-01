mod api;
mod components;
mod pages;
mod utils;

use components::header::*;
use leptos::*;
use leptos_router::*;
use pages::add_config_page::*;
use pages::add_label_page::*;
use pages::add_project_page::*;
use pages::apply_config_page::*;
use pages::config_list_page::*;
use pages::login_page::*;
use pages::modify_config_instance::*;
use pages::AdminPage;
use pages::LoginPage;
use std::collections::HashMap;
use yak_man_core::model::YakManRole;

use crate::pages::view_config_page::ViewConfigInstancePage;

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, <AppRouter/> })
}

#[derive(Clone, Debug)]
struct GlobalState {
    is_login_needed: bool,
    global_roles: Vec<YakManRole>,
    project_roles: HashMap<String, YakManRole>,
}

#[component]
pub fn AppRouter(cx: Scope) -> impl IntoView {
    let state = create_rw_signal(
        cx,
        GlobalState {
            is_login_needed: false,
            global_roles: vec![],
            project_roles: HashMap::new(),
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
                state.update(|s| {
                    s.global_roles = role_data.global_roles.clone();
                    s.project_roles = role_data.roles.clone();
                });
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

    let (is_login_needed, _) = create_slice(
        cx,
        state,
        |state| state.is_login_needed.clone(),
        |state, n| state.is_login_needed = n,
    );

    view! { cx,
        <Router>
            <Header/>
            <main>
                <Routes>
                    <Route
                        path="/login"
                        view=move |cx| {
                            view! { cx, <LoginPage/> }
                        }
                    />
                    <Route
                        path="/oauth-callback"
                        view=move |cx| {
                            view! { cx, <OauthCallbackPage/> }
                        }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/"
                        view=move |cx| {
                            view! { cx, <ConfigListPage/> }
                        }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/add-project"
                        view=move |cx| {
                            view! { cx, <AddProjectPage/> }
                        }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/admin"
                        view=move |cx| {
                            view! { cx, <AdminPage/> }
                        }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/add-config"
                        view=move |cx| {
                            view! { cx, <AddConfigPage/> }
                        }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/add-label"
                        view=move |cx| {
                            view! { cx, <AddLabelPage/> }
                        }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/apply/:config_name/:instance"
                        view=move |cx| {
                            view! { cx, <ApplyConfigPage/> }
                        }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/create-instance/:config_name"
                        view=move |cx| {
                            view! { cx, <CreateConfigInstancePage/> }
                        }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/edit-instance/:config_name/:instance"
                        view=move |cx| {
                            view! { cx, <EditConfigInstancePage/> }
                        }
                    />
                    <ProtectedRoute
                        condition=move |_| !is_login_needed()
                        redirect_path="/login"
                        path="/view-instance/:config_name/:instance"
                        view=move |cx| {
                            view! { cx, <ViewConfigInstancePage/> }
                        }
                    />
                </Routes>
            </main>
        </Router>
    }
}
