pub fn is_oauth_enabled() -> bool {
    return std::env::var("YAKMAN_OAUTH_ENABLED")
        .map(|v| v.parse::<bool>().ok())
        .ok()
        .flatten()
        .unwrap_or_default();
}

pub fn is_snapshot_backups_enabled() -> bool {
    return std::env::var("YAKMAN_SNAPSHOT_BACKUPS_ENABLED")
        .map(|v| v.parse::<bool>().ok())
        .ok()
        .flatten()
        .unwrap_or_default();
}

pub fn snapshot_backups_cron() -> String {
    return std::env::var("YAKMAN_SNAPSHOT_BACKUPS_CRON")
        .ok()
        .unwrap_or("0 0 * * * *".to_string());
}
