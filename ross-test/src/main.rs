use leptos::*;

use crate::{add_label_page::AddLabelPage, add_config_page::AddConfigPage};

mod add_label_page;
mod api;
mod add_config_page;

fn main() {
    mount_to_body(|cx| {
        view! { cx,
            <div>
                <AddConfigPage />
                // <AddLabelPage />
            </div>
        }
    })
}
