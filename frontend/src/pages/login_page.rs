use crate::api;
use crate::api::RequestError;
use crate::components::YakManButton;
use crate::components::YakManCard;
use crate::GlobalState;
use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use leptos::*;
use leptos_router::*;
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};

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
        <div class="container mx-auto">
            <YakManCard>
                <div class="flex flex-col items-center gap-4">
                    <h1 class="text-xl font-bold">{"Login"}</h1>
                    <a href=redirect_uri>
                        <YakManButton>"Click to login"</YakManButton>
                    </a>
                </div>
            </YakManCard>
        </div>
    }
}

#[component]
pub fn oauth_callback_page(cx: Scope) -> impl IntoView {
    let query = use_query_map(cx);
    let state = use_context::<RwSignal<GlobalState>>(cx).expect("state to have been provided");
    let (_, set_is_login_needed) = create_slice(
        cx,
        state,
        |state| state.is_login_needed,
        |state, n| state.is_login_needed = n,
    );

    let state = move || query.with(|params| params.get("state").cloned().unwrap());
    let code = move || query.with(|params| params.get("code").cloned().unwrap());

    let error_message = create_resource(
        cx,
        move || (),
        move |()| async move {
            let verifier = LocalStorage::raw()
                .get(LOCAL_STORAGE_OAUTH2_VERIFER_KEY)
                .unwrap()
                .map(|s| serde_json::from_str::<PkceCodeVerifier>(&s))
                .unwrap()
                .unwrap();

            let error = match api::exchange_oauth_code(&code(), &state(), verifier).await {
                Ok(_) => {
                    set_is_login_needed.set(false);
                    let navigate = use_navigate(cx);
                    navigate("/", Default::default()).unwrap();
                    None
                }
                Err(e) => match e {
                    RequestError::UnexpectedHttpStatus(status) if status == 403 => {
                        Some("Unauthorized User".to_string())
                    }
                    _ => {
                        error!("Token exchange failed {e}");
                        Some("Failed to login".to_string())
                    }
                },
            };

            error
        },
    );

    let error_message = move || error_message.read(cx).unwrap_or(None);

    view! { cx,
        <div style="display: flex; justify-content: center">
            <div>
                {move || match error_message() {
                    Some(error) => {
                        view! { cx, <>{error} <br/> <A href="/login">"Back to Login"</A></> }
                            .into_view(cx)
                    }
                    None => {
                        view! { cx, <h1>{"Logging in..."}</h1> }
                            .into_view(cx)
                    }
                }}
            </div>
        </div>
    }
}
