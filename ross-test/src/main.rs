use leptos::*;

use crate::{add_label_page::AddLabelPage, add_config_page::AddConfigPage, config_list_page::ConfigListPage};

mod add_label_page;
mod api;
mod add_config_page;
mod config_list_page;

fn main() {
    mount_to_body(|cx| {
        view! { cx,
            <div>
                <ConfigListPage />
                // <AddConfigPage />
                // <AddLabelPage />
            </div>
        }
    })
}
