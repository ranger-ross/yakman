use leptos::*;

#[component]
pub fn link_with_chrevon(cx: Scope, #[prop()] href: String, children: Children) -> impl IntoView {
    view! { cx,
        <a
            class="text-indigo-600 flex items-center text-lg"
            href={href}
        >
            {children(cx)}
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
            </svg>
        </a>
    }
}
