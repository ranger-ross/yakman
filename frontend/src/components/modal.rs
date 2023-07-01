use leptos::*;
use web_sys::MouseEvent;

use crate::components::button::YakManButtonVariant;
use crate::components::YakManButton;

#[component]
pub fn yak_man_modal<F>(
    cx: Scope,
    #[prop(into)] title: MaybeSignal<&'static str>,
    #[prop(into)] open: RwSignal<bool>,
    /// If true, clicking the backdrop will not close the modal. If false, clicking the backdrop will close the modal
    #[prop(into, default = false)]
    static_backdrop: bool,
    on_confirm: F,
    children: Children,
) -> impl IntoView
where
    F: Fn(MouseEvent) + 'static,
{
    let container_class = move || match open() {
        true => "opacity-100",
        false => "opacity-0 pointer-events-none",
    };

    let modal_class = move || match open() {
        true => "scale-100",
        false => "scale-95",
    };

    view! { cx,
        <div class=move || format!("absolute top-0 left-0 h-full w-full z-40 transition-opacity {}", container_class())>
            <div
                class="fixed z-10 inset-0 overflow-y-auto"
                aria-labelledby=title
                role="dialog"
                aria-modal="true"
            >
                <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
                    // Backdrop
                    <div
                        class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity {}"
                        aria-hidden="true"
                        on:click=move |_| {
                            if !static_backdrop {
                                open.set(false)
                            }
                        }
                    ></div>
                    // Spacer
                    <span
                        class="hidden sm:inline-block sm:align-middle sm:h-screen"
                        aria-hidden="true"
                    ></span>
                    // Modal Window
                    <div class=move || format!("inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full {}", modal_class())>
                        <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
                            <div class="sm:flex sm:items-start">
                                <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
                                    <h3 class="text-lg font-bold leading-6 text-gray-900">
                                        {title}
                                    </h3>
                                    <div class="mt-2">{children(cx)}</div>
                                </div>
                            </div>
                        </div>
                        <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
                            <YakManButton on:click=on_confirm >"Confirm"</YakManButton>
                            <YakManButton
                                variant=YakManButtonVariant::Secondary
                                on:click=move |_| open.set(false)
                            >
                                "Cancel"
                            </YakManButton>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
