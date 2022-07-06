use rocket::serde::uuid::Uuid;

pub fn create_uuid() -> String {
    Uuid::new_v4().to_string()
}
