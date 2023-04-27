use leptos::*;

use crate::add_label_page::AddLabelPage;

mod add_label_page;
mod api;

fn main() {
    mount_to_body(|cx| {
        view! { cx,
            <div>
                <AddLabelPage />
            </div>
        }
    })
}
