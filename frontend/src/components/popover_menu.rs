use std::time::Duration;

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

const OPENED_CLASSES: &str = "transform opacity-100 scale-100";
const CLOSED_CLASSES: &str = "transform opacity-0 scale-95";

#[component]
pub fn popover_menu(cx: Scope, #[prop()] options: Vec<PopoverMenuOption>) -> impl IntoView {
    let (open, set_open) = create_signal(cx, false);
    let (extra_class, set_extra_class) = create_signal(cx, CLOSED_CLASSES);

    let on_change = move |is_open: bool| {
        if is_open {
            set_open(is_open);

            // Add a slight delay to wait for the elements to be added to the DOM 
            // so that when the classes are added the animation plays properly
            set_timeout(
                move || {
                    set_extra_class(OPENED_CLASSES);
                },
                Duration::from_millis(1),
            );
        } else {
            set_extra_class(CLOSED_CLASSES);
            set_timeout(
                move || {
                    set_open(is_open);
                },
                // This duration should match the duration Tailwind class below
                Duration::from_millis(100),
            );
        }
    };

    view! { cx,
        <div class="relative inline-block text-left">
            <KebabMenuIcon
                class="cursor-pointer"
                on:click=move |_| on_change(!open())
                on:blur=move |_| on_change(false)
            />

            <Show
                when=open
                fallback=|_| view! { cx, }
            >
                <div class=move || format!("origin-top-right absolute right-0 mt-2 w-56 rounded-md shadow-lg ring-1 ring-black bg-white ring-opacity-5 transition ease-out duration-100 {}", extra_class())>
                    <div
                        class="py-1"
                        role="menu"
                        aria-orientation="vertical"
                        aria-labelledby="options-menu"
                    >
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
            </Show>

          </div>
    }
}
