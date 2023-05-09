use leptos::*;
use leptos_router::*;
use yak_man_core::model::YakManRole;

use crate::GlobalState;

#[component]
pub fn header(cx: Scope) -> impl IntoView {
    let state = use_context::<RwSignal<GlobalState>>(cx).expect("state to have been provided");
    let (global_roles, _) = create_slice(
        cx,
        state,
        |state| state.global_roles.clone(),
        |state, n| state.global_roles = n,
    );

    let is_admin = move || global_roles().contains(&YakManRole::Admin);

    view! { cx,
        <div style="display: flex; justify-content: end; gap: 10px; margin-bottom: 0.5rem; padding: 8px; border-bottom: solid 1px darkgray">
            <a href="/">{"YakMan"}</a>
            <div style="flex-grow: 1" />

            <a href="/login">{"Login"}</a>
            <a href="/add-config">{"Add Config"}</a>
            <a href="/add-label">{"Add Label"}</a>
            
            <Show
                when=is_admin
                fallback=|_| view! { cx,  }
            >
                <a href="/add-project">{"Add Project"}</a>
                <a href="/admin">{"Admin"}</a>
            </Show>
  
        </div>
    }
}
