pub mod local_file_adapter;
mod storage_types;

pub use self::local_file_adapter::LocalFileStorageAdapter;
use super::{KeyValuePairStorageAdapter, GenericStorageError};

pub fn create_local_file_adapter() -> LocalFileStorageAdapter {
    return LocalFileStorageAdapter {
        path: "/home/ross/projects/config-manager/testing-directory".to_string(),
        yakman_dir: None
    };
}
