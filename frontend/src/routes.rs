use yew_router::Routable;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/add-config")]
    AddConfigPage,
    #[at("/add-label")]
    AddLabelPage,
    #[at("/create-instance/:config_name")]
    CreateConfigInstancePage { config_name: String },
    #[at("/edit-instance/:config_name/:instance")]
    EditConfigInstancePage {
        config_name: String,
        instance: String,
    },
    #[at("/history/:config_name/:instance")]
    RevisionHistoryPage {
        config_name: String,
        instance: String,
    },
    #[not_found]
    #[at("/404")]
    NotFound,
}