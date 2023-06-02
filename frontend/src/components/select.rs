use std::borrow::Cow;

use crate::components::ArrowDownIcon;
use leptos::*;

#[component]
pub fn yak_man_select(
    cx: Scope,
    #[prop(into)] label: MaybeSignal<Cow<'static, str>>,
    children: Children,
) -> impl IntoView {
    view! { cx,
        <div class="w-64">
            <label class="block text-gray-700 text-sm font-bold mb-2">{label}</label>
            <div class="relative">
                <select class="block appearance-none w-full bg-white border border-gray-400 hover:border-indigo-500 px-4 py-2 pr-8 rounded shadow leading-tight focus:outline-none focus:shadow-outline">
                    {children(cx)}
                </select>
                <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700">
                    <ArrowDownIcon />
                </div>
            </div>
        </div>
    }
}
