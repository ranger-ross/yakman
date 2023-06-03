use leptos::*;

#[component]
pub fn yak_man_text_area(
    cx: Scope,
    #[prop(into)] label: MaybeSignal<&'static str>,
    #[prop(into)] value: MaybeSignal<String>,
    #[prop(optional, into)] placeholder: MaybeSignal<&'static str>,
) -> impl IntoView {
    view! { cx,
        <div class="w-64">
            <label class="block text-gray-700 text-sm font-bold mb-2">{label}</label>
            <div class="relative">
                <textarea
                    class="block appearance-none w-full bg-white border border-gray-400 hover:border-indigo-500 px-4 py-2 pr-8 rounded shadow leading-tight focus:outline-none focus:shadow-outline transition-all duration-200" 
                    rows="4"
                    placeholder=placeholder
                    prop:value=value
                ></textarea>
            </div>
        </div>
    }
}