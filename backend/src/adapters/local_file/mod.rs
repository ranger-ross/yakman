pub mod local_file_adapter;
mod storage_types;

pub use self::local_file_adapter::LocalFileStorageAdapter;
use super::{GenericStorageError, KVStorageAdapter};
