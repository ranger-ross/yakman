use crate::api;
use leptos::*;
use yak_man_core::model::{YakManRole, YakManUser};

#[component]
pub fn admin_page(cx: Scope) -> impl IntoView {
    let new_username = create_rw_signal(cx, String::from(""));

    let users = create_resource(
        cx,
        move || (),
        |()| async move { api::fetch_users().await.unwrap() },
    );

    let users = move || users.read(cx).unwrap_or(vec![]);

    let create_user = move |_| {
        spawn_local(async move {
            let new_username = new_username.get();
            for user in users() {
                if user.email == new_username {
                    log!("user already added, skipping...");
                    return;
                }
            }

            api::create_user(&new_username, &YakManRole::Viewer)
                .await
                .unwrap();
        })
    };

    view! { cx,
        <div>
            <h1>{"Admin"}</h1>
            <h2>{"Users"}</h2>
            <For
                each=users
                key=|user| user.email.clone()
                view=move |cx, user: YakManUser| {
                    view! { cx, <p>{user.email}</p> }
                }
            />
            <h2>{"Add User"}</h2>
            {"Username "}
            <input
                on:input=move |ev| new_username.set(event_target_value(&ev))
                prop:value=move || new_username.get()
            />
            <br/>
            <button on:click=create_user>{"Create user"}</button>
        </div>
    }
}
