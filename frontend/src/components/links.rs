use crate::components::ArrowRightIcon;
use leptos::*;

#[component]
pub fn link_with_chrevon(cx: Scope, #[prop()] href: String, children: Children) -> impl IntoView {
    view! { cx,
        <a
            class="text-indigo-600 flex items-center text-lg"
            href={href}
        >
            {children(cx)}
            <ArrowRightIcon />
        </a>
    }
}
