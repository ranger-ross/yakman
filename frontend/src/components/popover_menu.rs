use leptos::*;

use crate::components::KebabMenuIcon;

#[derive(Debug, PartialEq, Clone)]
pub struct PopoverMenuOption {
    pub href: String,
    pub text: String,
}

impl PopoverMenuOption {
    pub fn new(href: &str, text: &str) -> PopoverMenuOption {
        PopoverMenuOption {
            text: String::from(text),
            href: String::from(href),
        }
    }
}

#[component]
pub fn popover_menu(
    cx: Scope,
    #[prop()] options: Vec<PopoverMenuOption>,
) -> impl IntoView
{
    let (open, set_open) = create_signal(cx, false);

    view! { cx,
        <div class="relative inline-block text-left">
            <KebabMenuIcon
                class="cursor-pointer"
                on:click=move |_| set_open(!open())
            />

            // TODO: impl this animation
            // <!--
            //   Dropdown menu, show/hide based on menu state.
            // 
            //   Entering: "transition ease-out duration-100"
            //     From: "transform opacity-0 scale-95"
            //     To: "transform opacity-100 scale-100"
            //   Leaving: "transition ease-in duration-75"
            //     From: "transform opacity-100 scale-100"
            //     To: "transform opacity-0 scale-95"
            // -->
            {move || if open() {
               view! { cx,
                    <div class="origin-top-right absolute right-0 mt-2 w-56 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5">
                        <div class="py-1" role="menu" aria-orientation="vertical" aria-labelledby="options-menu">
                            {options.iter().map(|option| {
                                view! { cx,
                                    <a
                                        href={&option.href}
                                        class="cursor-pointer block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900"
                                        role="menuitem"
                                    >
                                        {&option.text}
                                    </a>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
               }
            } else {
                view! { cx,
                    <div></div>
               }
            }}

          </div>
    }
}
