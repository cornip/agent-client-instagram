use aes_gcm::{
    aead::KeyInit,
    Aes256Gcm, Nonce, Key,
    AeadInPlace
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey, pkcs8::DecodePublicKey};
use std::time::{SystemTime, UNIX_EPOCH};
use base32::decode;
use base32::Alphabet::RFC4648;
use totp_rs::{Algorithm, TOTP};
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;

pub const BLOKS_VERSION: &str = "16e9197b928710eafdf1e803935ed8c450a1a2e3eb696bff1184df088b900bcf";
pub const USER_AGENT: &str = "Instagram 361.0.0.46.88 Android (28/9; 240dpi; 720x1280; Asus; ASUS_I003DD; ASUS_I003DD; intel; en_US; 674675155)";
pub async fn encrypt_password(password: &str) -> String {
    let (public_key, public_key_id) = get_public_keys().await;
    let session_key = rand::random::<[u8; 32]>();
    let iv = rand::random::<[u8; 12]>();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let decoded_key = BASE64.decode(public_key).unwrap();
    let public_key = RsaPublicKey::from_public_key_pem(std::str::from_utf8(&decoded_key).unwrap()).unwrap();


    let rsa_encrypted = public_key
        .encrypt(&mut rand::thread_rng(), Pkcs1v15Encrypt, &session_key)
        .unwrap();

    let key = Key::<Aes256Gcm>::from_slice(&session_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&iv);
    
    let mut buffer = password.as_bytes().to_vec();
    let tag = cipher
        .encrypt_in_place_detached(
            nonce,
            timestamp.as_bytes(),
            &mut buffer
        )
        .unwrap();

    let size_buffer = (rsa_encrypted.len() as u16).to_le_bytes();
    let mut payload = Vec::new();
    payload.push(1u8);
    payload.push(public_key_id as u8);
    payload.extend_from_slice(&iv);
    payload.extend_from_slice(&size_buffer);
    payload.extend_from_slice(&rsa_encrypted);
    payload.extend_from_slice(&tag);
    payload.extend_from_slice(&buffer);

    format!(
        "#PWD_INSTAGRAM:4:{}:{}",
        timestamp,
        BASE64.encode(&payload)
    )
}

async fn get_public_keys() -> (String, u8) {
    let client = Client::new();
    let response = client
        .get("https://i.instagram.com/api/v1/qe/sync/")
        .send()
        .await
        .unwrap();

    let public_key = response
        .headers()
        .get("ig-set-password-encryption-pub-key")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let public_key_id = response
        .headers()
        .get("ig-set-password-encryption-key-id")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u8>()
        .unwrap();

    (public_key, public_key_id)
} 
pub fn extract_bearer_token(response: String) -> Option<String> {
    let pattern = r#"Bearer IGT:2:[^\\"}\s]+"#;
    let re = regex::Regex::new(pattern).unwrap();
    let result = re.find(&response)
        .map(|m| m.as_str().to_string());
    result
}
pub fn generate_totp_code(secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    let decoded = decode(RFC4648 { padding: false }, secret)
        .ok_or("Failed to decode TOTP secret")?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        decoded,
    ).map_err(|e| format!("Failed to create TOTP: {}", e))?;

    totp.generate_current()
        .map_err(|e| format!("Failed to generate TOTP code: {}", e).into())
}
pub fn extract_two_factor_context(response: String) -> String {
    let re = regex::Regex::new(r#"AW[\w-]+"#).unwrap();
    re.find(&response)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default()
}
pub fn generate_device_id() -> String {
    use rand::{thread_rng, Rng};
    const CHARSET: &[u8] = b"0123456789abcdef";
    let mut rng = thread_rng();
    
    let mut result = String::from("android-");
    for _ in 0..16 {
        let idx = rng.gen_range(0..CHARSET.len());
        result.push(CHARSET[idx] as char);
    }
    result
}
pub fn generate_uuid() -> String {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    
    let mut uuid = String::with_capacity(36);
    for i in 0..36 {
        match i {
            8 | 13 | 18 | 23 => uuid.push('-'),
            14 => uuid.push('4'), 
            19 => {
                let n = rng.gen_range(0..16);
                uuid.push(char::from_digit((n & 0x3) | 0x8, 16).unwrap())
            }
            _ => {
                let n = rng.gen_range(0..16);
                uuid.push(char::from_digit(n, 16).unwrap())
            }
        }
    }
    uuid
}
pub fn generate_random_key() -> String {
    use rand::{thread_rng, Rng};
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = thread_rng();
    
    (0..28)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn gen_user_breadcrumb(size: u32) -> String {
    let key = b"iN4$aGr0m";
    let dt = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let mut rng = rand::thread_rng();
    
    let time_elapsed = rng.gen_range(500..1500) + size * rng.gen_range(500..1500);
    
    let text_change_event_count = f64::max(1.0, size as f64 / rng.gen_range(3..5) as f64);
    
    let data = format!(
        "{} {} {} {}", 
        size, 
        time_elapsed, 
        text_change_event_count, 
        dt
    );

    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize().into_bytes();

    format!(
        "{}\n{}\n",
        BASE64.encode(result),
        BASE64.encode(data.as_bytes())
    )
}

pub fn generate_numeric_id() -> String {
    let mut rng = rand::thread_rng();
    let mut id = String::with_capacity(11);
    id.push(char::from_digit(rng.gen_range(1..10), 10).unwrap());
    for _ in 0..10 {
        id.push(char::from_digit(rng.gen_range(0..10), 10).unwrap());
    }
    
    id
}
