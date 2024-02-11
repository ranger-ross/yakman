pub fn is_oauth_enabled() -> bool {
    return std::env::var("YAKMAN_OAUTH_ENABLED")
        .map(|v| v.parse::<bool>().ok())
        .ok()
        .flatten()
        .unwrap_or_default();
}

pub fn is_snapshot_backups_enabled() -> bool {
    return std::env::var("YAKMAN_ENABLE_SNAPSHOT_BACKUPS")
        .map(|v| v.parse::<bool>().ok())
        .ok()
        .flatten()
        .unwrap_or_default();
}
