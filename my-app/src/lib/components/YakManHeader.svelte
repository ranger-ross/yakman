<div
    class="bg-white shadow-sm h-14 flex justify-end items-center gap-3 mb-2 p-2"
>
    <a class="text-2xl font-bold" href="/">YakMan</a>
    <div style="flex-grow: 1" />
    <a href="/login">Login</a>
    <a href="/add-config">Add Config</a>
    <a href="/add-label">Add Label</a>
    <a href="/add-project">Add Project</a>
    <a href="/admin">Admin</a>
    <!-- TODO: Handle is Admin -->
    <!-- <Show
        when=is_admin
        fallback=|_| {
            view! { cx,  }
        }
    >
        <a href="/add-project">{"Add Project"}</a>
        <a href="/admin">{"Admin"}</a>
    </Show> -->
</div>

<!-- use leptos::*;
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

    let is_admin = move || global_roles.get().contains(&YakManRole::Admin);

    view! { cx,
        <div class="bg-white shadow-sm h-14 flex justify-end items-center gap-3 mb-2 p-2">
            <a class="text-2xl font-bold" href="/">
                {"YakMan"}
            </a>
            <div style="flex-grow: 1"></div>
            <a href="/login">{"Login"}</a>
            <a href="/add-config">{"Add Config"}</a>
            <a href="/add-label">{"Add Label"}</a>
            <Show
                when=is_admin
                fallback=|_| {
                    view! { cx,  }
                }
            >
                <a href="/add-project">{"Add Project"}</a>
                <a href="/admin">{"Admin"}</a>
            </Show>
        </div>
    }
}
 -->
