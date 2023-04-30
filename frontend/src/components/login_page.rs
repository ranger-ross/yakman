use leptos::*;

use crate::api;

#[component]
pub fn login_page(cx: Scope) -> impl IntoView {
    let data = create_resource(
        cx,
        move || (),
        |()| async move {
            let uri = api::fetch_oauth_redirect_uri().await;
            uri.expect("failed to get oauth redirect uri")
        },
    );

    let redirect_uri = move || data.read(cx).unwrap_or(String::new());

    view! { cx,
        <div>
            <h1>{"Login"}</h1>

            <a href=redirect_uri>"Click to login"</a>

        </div>
    }
}
