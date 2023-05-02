use std::collections::HashMap;

use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use leptos::*;
use leptos_router::use_navigate;
use leptos_router::{use_params_map, use_query_map};
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};
use serde::Serialize;

use crate::api;

const LOCAL_STORAGE_OAUTH2_VERIFER_KEY: &str = "oauth2-verifier";

#[component]
pub fn login_page(cx: Scope) -> impl IntoView {
    let data = create_resource(
        cx,
        move || (),
        |()| async move {
            let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
            let verifier = serde_json::to_string(&pkce_verifier).unwrap();

            LocalStorage::raw()
                .set(LOCAL_STORAGE_OAUTH2_VERIFER_KEY, &verifier)
                .unwrap();

            let uri = api::fetch_oauth_redirect_uri(pkce_challenge).await;
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

#[component]
pub fn oauth_callback_page(cx: Scope) -> impl IntoView {
    let query = use_query_map(cx);

    let state = move || query.with(|params| params.get("state").cloned().unwrap());
    let code = move || query.with(|params| params.get("code").cloned().unwrap());

    create_resource(
        cx,
        move || (),
        move |()| async move {

            log!("Call create_resource");

            let verifier = LocalStorage::raw()
                .get(LOCAL_STORAGE_OAUTH2_VERIFER_KEY)
                .unwrap()
                .map(|s| serde_json::from_str::<PkceCodeVerifier>(&s))
                .unwrap()
                .unwrap();

            let x = api::exchange_oauth_code(&code(), &state(), verifier)
                .await
                .unwrap();

            log!("Exchange complete: d: {x} len: {}", x.len());

            let navigate = use_navigate(cx);
            navigate("/", Default::default()).unwrap();
        },
    );

    view! { cx,
        <div>
            <h1>{"Callback page (loading....)"}</h1>

            {state} <br />
            {code} <br />
      
        </div>
    }
}
