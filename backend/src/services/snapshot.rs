use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    adapters::{errors::GenericStorageError, KVStorageAdapter},
    model::YakManSnapshotLock,
};

struct SnapshotService {
    pub adapter: Box<dyn KVStorageAdapter>,
}

impl SnapshotService {
    pub async fn take_snapshot(&self) {
        if let Some(lock) = self.try_take_lock().await {
            log::info!("Aquired snapshot lockfile");
            todo!();
        } else {
            log::debug!("Snapshot lock already taken");
        }
    }

    async fn try_take_lock(&self) -> Option<YakManSnapshotLock> {
        let snapshot_lock = self.adapter.get_snapshot_lock().await;

        let taken_lock = if let Some(lock) = snapshot_lock.map(|s| s.lock).ok()? {
            // Lock already taken, but check if its expired
            // in the event that the previous snapshot failed and the lock is permanently taken
            let expiration_timestamp = Utc::now() + Duration::minutes(30);
            if lock.timestamp_ms < expiration_timestamp.timestamp_millis() {
                self.create_new_lock()
            } else {
                // Lock is not expired
                return None;
            }
        } else {
            self.create_new_lock()
        };

        if let Err(err) = self.adapter.save_snapshot_lock(&taken_lock).await {
            log::error!("Failed to save lock. Error: {err:?}");
            return None;
        }

        // Since there are multiple types for storage systems there is no way to take an atomic lock.
        // So wait few seconds and recheck the lock file to make sure it was not overwritten by another instance
        let sleep_duration = Duration::seconds(10).to_std().unwrap();
        tokio::time::sleep(sleep_duration).await;
        let inner = &taken_lock
            .lock
            .as_ref()
            .expect("Lock is created above so it will never be None");

        if let Ok(Some(lock)) = self.adapter.get_snapshot_lock().await.map(|s| s.lock) {
            if lock.id != inner.id {
                log::warn!("Lock was overriden, bailing");
                return None;
            }
        } else {
            return None;
        }

        return Some(taken_lock);
    }

    fn create_new_lock(&self) -> YakManSnapshotLock {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();
        YakManSnapshotLock::new(id, now)
    }
}
