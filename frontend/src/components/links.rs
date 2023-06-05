use crate::components::ArrowRightIcon;
use leptos::*;

#[component]
pub fn link_with_chrevon(cx: Scope, #[prop()] href: String, children: Children) -> impl IntoView {
    view! { cx,
        <a
            class="text-indigo-600 hover:text-indigo-800 flex items-center text-lg transition-colors duration-200"
            href=href
        >
            {children(cx)}
            <ArrowRightIcon/>
        </a>
    }
}
