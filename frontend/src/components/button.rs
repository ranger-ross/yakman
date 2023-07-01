use leptos::*;

#[derive(Clone)]
pub enum YakManButtonVariant {
    Primary,
    Secondary,
}

impl Default for YakManButtonVariant {
    fn default() -> Self {
        YakManButtonVariant::Primary
    }
}

#[component]
pub fn yak_man_button(
    cx: Scope,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional)] variant: YakManButtonVariant,
    children: Children,
) -> impl IntoView {
    let disabled = disabled.unwrap_or(false);
    let button_css_classes = move || {
        match variant {
            YakManButtonVariant::Primary => "bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-300 text-white text-lg font-bold rounded-md shadow-md py-1 px-4 m-1 transition-colors duration-200",
            YakManButtonVariant::Secondary => "bg-white hover:bg-gray-50 disabled:bg-gray-300 rounded-md shadow-sm py-1 px-4 m-1 text-lg text-gray-700 border border-gray-300 transition-colors duration-200",
        }
    };

    view! { cx,
        <button disabled=disabled class=button_css_classes>
            {children(cx)}
        </button>
    }
}
