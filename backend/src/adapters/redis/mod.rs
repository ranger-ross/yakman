use self::redis_adapter::RedisStorageAdapter;
use super::KVStorageAdapter;

pub mod redis_adapter;

pub fn create_redis_adapter() -> RedisStorageAdapter {
    // TODO: use env vars
    return RedisStorageAdapter {
        host: "127.0.0.1".to_string(),
        port: 6379,
        username: "".to_string(),
        password: "".to_string(),
    };
}

