use crate::api;
use leptos::*;
use yak_man_core::model::{YakManRole, YakManUser};

#[component]
pub fn admin_page(cx: Scope) -> impl IntoView {
    let (new_username, set_new_username) = create_signal(cx, String::from(""));

    let users = create_resource(
        cx,
        move || (),
        |()| async move { api::fetch_users().await.unwrap() },
    );

    let users = move || users.read(cx).unwrap_or(vec![]);

    let create_user = create_action(cx, move |_: &()| async move {
        for user in users() {
            if user.email == new_username() {
                log!("user already added, skipping...");
                return;
            }
        }

        api::create_user(&new_username(), &YakManRole::Viewer)
            .await
            .unwrap();
    });

    view! { cx,
        <div>
            <h1>{"Admin"}</h1>

            <h2>{"Users"}</h2>

            <For
                each=users
                key=|user| user.email.clone()
                view=move |cx, user: YakManUser| view! { cx,
                    <p>{user.email} " => " {user.role.to_string()} </p>
                }
            />

            <h2>{"Add User"}</h2>
            {"Username "}<input on:input=move |ev| set_new_username(event_target_value(&ev)) prop:value=new_username  />
            <br />

            <button on:click=move |_| create_user.dispatch(())>
                {"Create user"}
            </button>


        </div>
    }
}
