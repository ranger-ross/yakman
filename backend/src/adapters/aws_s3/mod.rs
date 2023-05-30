pub mod aws_s3_adapter;
mod storage_types;

pub use self::aws_s3_adapter::AwsS3StorageAdapter;
use super::{GenericStorageError, KVStorageAdapter};

use aws_sdk_s3 as s3;

pub async fn create_aws_s3_adapter() -> AwsS3StorageAdapter {
    let config = ::aws_config::load_from_env().await;
    let client = s3::Client::new(&config);

    AwsS3StorageAdapter {
        yakman_dir: None,
        client: client,
        bucket: "yakman-testing".to_string()
    }
}
