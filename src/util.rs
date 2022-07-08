use qrcode_generator::QrCodeEcc;
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

pub fn generate_qr(payment_request: &str) -> Vec<u8> {
    qrcode_generator::to_png_to_vec(payment_request, QrCodeEcc::Low, 128).unwrap()
}

pub fn to_hex(bytes: &Vec<u8>) -> String {
    hex::encode(bytes)
}

pub fn from_hex(hex_str: &str) -> Vec<u8> {
    hex::decode(hex_str).unwrap()
}

pub fn to_base64(bytes: &Vec<u8>) -> String {
    base64::encode(bytes)
}
