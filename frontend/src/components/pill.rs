use leptos::*;

#[component]
pub fn status_pill(cx: Scope, children: Children) -> impl IntoView {
    view! { cx,
        <div class="bg-yellow-100 text-yellow-900 text-md rounded-full pl-2 pr-2 pt-1 pb-1">
            {children(cx)}
        </div>
    }
}
