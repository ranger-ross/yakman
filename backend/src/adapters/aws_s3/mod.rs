pub mod aws_s3_adapter;
mod storage_types;

pub use self::aws_s3_adapter::AwsS3StorageAdapter;
use super::{GenericStorageError, KVStorageAdapter};
