use leptos::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn color_from_str(input: &str) -> (String, String) {
    // Define a small array of Tailwind color classes for background and text
    let colors = [
        ("bg-red-100", "text-red-900"),
        ("bg-yellow-100", "text-yellow-900"),
        ("bg-green-100", "text-green-900"),
        ("bg-blue-100", "text-blue-900"),
        ("bg-indigo-100", "text-indigo-900"),
        ("bg-purple-100", "text-purple-900"),
        ("bg-pink-100", "text-pink-900"),
    ];

    // Create a new hasher
    let mut hasher = DefaultHasher::new();
    
    // Write our input into the hasher
    input.hash(&mut hasher);
    
    // Obtain the hash value
    let hash = hasher.finish();
    
    // Map the hash value to one of the color pairs
    let (bg_color_class, text_color_class) = colors[(hash as usize) % colors.len()];

    (bg_color_class.to_string(), text_color_class.to_string())
}
#[component]
pub fn label_pill(cx: Scope, #[prop()] text: String) -> impl IntoView {
    let (background, text_color) = color_from_str(&text);
    view! { cx,
        <div class={format!("{background} {text_color} text-sm rounded-full pl-2 pr-2 pt-1 pb-1 w-fit")}>
            {text}
        </div>
    }
}

#[component]
pub fn status_pill(cx: Scope, children: Children) -> impl IntoView {
    view! { cx,
        <div class="bg-yellow-100 text-yellow-900 text-md rounded-full pl-2 pr-2 pt-1 pb-1">
            {children(cx)}
        </div>
    }
}
