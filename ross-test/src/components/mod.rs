pub mod modify_config_instance;
// pub mod revision_history;
pub mod config_list_page;
pub mod add_config_page;
pub mod add_label_page;
pub mod apply_config_page;

pub use modify_config_instance::CreateConfigInstancePage;
pub use modify_config_instance::EditConfigInstancePage;
// pub use revision_history::RevisionHistoryPage;
pub use config_list_page::ConfigListPage;
pub use add_config_page::AddConfigPage;
pub use add_label_page::AddLabelPage;
pub use apply_config_page::ApplyConfigPage;