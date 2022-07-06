use rocket::serde::uuid::Uuid;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub fn create_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
