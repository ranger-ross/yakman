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

pub fn yakman_application_host() -> Option<String> {
    return std::env::var("YAKMAN_APPLICATION_HOST").ok();
}

pub fn is_notifications_enabled() -> bool {
    let r = std::env::var("YAKMAN_NOTIFICATIONS_ENABLED")
        .map(|v| v.parse::<bool>().ok())
        .ok()
        .flatten()
        .unwrap_or_default();

    log::info!("ENABLED {r}");

    return r;
}

pub fn notification_whitelisted_hosts() -> Vec<String> {
    return from_comma_delimited_list("YAKMAN_NOTIFICATION_WEBHOOK_HOSTS");
}

fn from_comma_delimited_list(env_var_name: &str) -> Vec<String> {
    let env_var = match std::env::var(env_var_name) {
        Ok(val) => val,
        Err(_) => return vec![],
    };

    return env_var
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()) // Filter out empty strings
        .collect();
}
