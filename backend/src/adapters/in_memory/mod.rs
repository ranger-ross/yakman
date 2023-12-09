pub mod in_memory_adapter;
mod storage_types;

pub use self::in_memory_adapter::InMemoryStorageAdapter;
use super::{KVStorageAdapter, GenericStorageError};

