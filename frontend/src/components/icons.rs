use leptos::*;

#[component]
pub fn home_icon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg class="q m" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path>
        </svg>
    }
}

#[component]
pub fn arrow_down_icon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" class="h-4 w-4">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
        </svg>
    }
}

#[component]
pub fn arrow_right_icon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
        </svg>
    }
}

#[component]
pub fn kebab_menu_icon<'a>(cx: Scope, #[prop(optional)] class: &'a str) -> impl IntoView {
    view! { cx,
        <svg class={&format!("w-5 h-5 focus:outline-none {}", class)} viewBox="0 0 24 24" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" fill="#000000" data-darkreader-inline-fill="" style="--darkreader-inline-fill: #000000;"><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"> <title>"Kebab-Menu"</title> <g id="Kebab-Menu" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd" data-darkreader-inline-stroke="" style="--darkreader-inline-stroke: none;"> <rect id="Container" x="0" y="0" width="24" height="24"> </rect> <path d="M12,6 C12.5522847,6 13,5.55228475 13,5 C13,4.44771525 12.5522847,4 12,4 C11.4477153,4 11,4.44771525 11,5 C11,5.55228475 11.4477153,6 12,6 Z" id="shape-03" stroke="#030819" stroke-width="2" stroke-linecap="round" stroke-dasharray="0,0" data-darkreader-inline-stroke="" style="--darkreader-inline-stroke: #dfdcd8;"> </path> <path d="M12,13 C12.5522847,13 13,12.5522847 13,12 C13,11.4477153 12.5522847,11 12,11 C11.4477153,11 11,11.4477153 11,12 C11,12.5522847 11.4477153,13 12,13 Z" id="shape-03" stroke="#030819" stroke-width="2" stroke-linecap="round" stroke-dasharray="0,0" data-darkreader-inline-stroke="" style="--darkreader-inline-stroke: #dfdcd8;"> </path> <path d="M12,20 C12.5522847,20 13,19.5522847 13,19 C13,18.4477153 12.5522847,18 12,18 C11.4477153,18 11,18.4477153 11,19 C11,19.5522847 11.4477153,20 12,20 Z" id="shape-03" stroke="#030819" stroke-width="2" stroke-linecap="round" stroke-dasharray="0,0" data-darkreader-inline-stroke="" style="--darkreader-inline-stroke: #dfdcd8;"> </path> </g> </g></svg>
    }
}
