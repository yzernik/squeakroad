use qr_code::QrCode;
use rocket::serde::uuid::Uuid;
use std::io::Cursor;
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
    let qr = QrCode::new(&payment_request.as_bytes()).unwrap();
    let bmp = qr.to_bmp().add_white_border(4).unwrap().mul(4).unwrap();
    let mut cursor = Cursor::new(vec![]);
    bmp.write(&mut cursor).unwrap();
    cursor.into_inner()
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
