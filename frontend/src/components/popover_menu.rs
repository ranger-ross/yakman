use crate::components::KebabMenuIcon;
use leptos::*;
use std::time::Duration;

#[derive(Debug, PartialEq, Clone)]
pub struct PopoverMenuOption<T> {
    pub value: T,
    pub text: String,
}

impl<T> PopoverMenuOption<T> {
    pub fn new(value: T, text: &str) -> PopoverMenuOption<T> {
        PopoverMenuOption {
            value: value,
            text: String::from(text),
        }
    }
}

const OPENED_CLASSES: &str = "transform opacity-100 scale-100";
const CLOSED_CLASSES: &str = "transform opacity-0 scale-95";

#[component]
pub fn popover_menu<F, T>(
    cx: Scope,
    #[prop()] options: Vec<PopoverMenuOption<T>>,
    #[prop()] on_select: F,
) -> impl IntoView
where
    T: Clone + 'static,
    F: Fn(&T) + 'static + Clone,
{
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
                fallback=|_| {
                    view! { cx,  }
                }
            >
                <div class=move || {
                    format!(
                        "origin-top-right absolute right-0 mt-2 w-56 rounded-md shadow-lg ring-1 ring-black bg-white ring-opacity-5 transition ease-out duration-100 {}",
                        extra_class()
                    )
                }>
                    <div
                        class="py-1"
                        role="menu"
                        aria-orientation="vertical"
                        aria-labelledby="options-menu"
                    >


                    {options
                        .clone()
                        .into_iter()
                        .map(|option| {
                            let on_select = on_select.clone();
                            let value = option.value.clone();
                            view! { cx,
                                <a
                                    class="cursor-pointer block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900"
                                    role="menuitem"
                                    on:click=move |_| on_select(&value)
                                >
                                    {option.text}
                                </a>
                            }
                        })
                        .collect::<Vec<_>>()}
                    </div>
                </div>
            </Show>
        </div>
    }
}
