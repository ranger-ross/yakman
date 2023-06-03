use leptos::*;

#[component]
pub fn yak_man_button(
    cx: Scope,
    #[prop(optional)] disabled: Option<bool>,
    children: Children,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    view! { cx,
        <button disabled={disabled} class="bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-300 text-white text-lg font-bold rounded-lg shadow-md py-1 px-4 m-1 transition-colors duration-200">{children(cx)}</button>
    }
}
