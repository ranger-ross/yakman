use leptos::*;

#[component]
pub fn yak_man_card(cx: Scope, children: Children) -> impl IntoView {
    view! { cx,
        <div class="bg-white w-full p-4 rounded shadow-sm">
            {children(cx)}
        </div>
    }
}
