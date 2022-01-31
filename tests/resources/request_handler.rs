use reqwest;

/// Assembles and sends a request to private api.
/// The function supports 2FA and needs to be given an OTP secret key.
/// The procedure of assembling a get request to private API consists of:
/// * Preparing "nonce" and "otp" values, which are contained in key-value vector
/// * Using the vector to create url encoded payload
/// * Using the paylaod and nonce value along with the private key to create the signature
/// * Sending the request using public key and signature as headers and url encoded payload as body
///
/// # Arguments
///
/// * `api_key` - Public key for API
/// * `api_secret` - Private key for API
/// * `otp_secret` - Secret serving as private key to generate one time password
/// * `api_link` - Basic link to API, without any predefined endpoint
/// * `endpoint_path` - Specific endpoint which is the target of sent requests
///
pub async fn private_api_request(
    api_key: &str,
    api_secret: &str,
    otp_secret: &str,
    api_link: &str,
    endpoint_path: &str,
) -> reqwest::Result<reqwest::Response> {
    let otp = properties::get_otp_code(&otp_secret.into());
    let nonce = properties::get_nonce();
    let body_data: Vec<(&str, &str)> = vec![("nonce", &nonce), ("otp", &otp)];
    let url_encoded_payload: String = url_encoding::url_encode(&body_data);
    let signature =
        encryption::get_signature(&nonce, &url_encoded_payload, endpoint_path, api_secret);
    let full_link = [api_link, endpoint_path].concat();

    let result =
        requesting::send_request(&full_link, &url_encoded_payload, api_key, &signature).await;
    result
}

mod properties {
    use boringauth::oath::TOTPBuilder;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Creates otp code from otp secret.
    ///
    /// # Arguments
    ///
    /// * `otp_secret` - Secret serving as private key to generate one time password
    ///
    pub fn get_otp_code(otp_secret: &String) -> String {
        let code = TOTPBuilder::new()
            .base32_key(otp_secret)
            .finalize()
            .expect("Incorrect OTP secret")
            .generate();
        code
    }

    /// Creates nonce from current time timestamp;
    /// It serves as a value that increases across the request sent to API.
    pub fn get_nonce() -> String {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time interval from unix epoch should be positive");
        since_the_epoch.as_millis().to_string()
    }
}

mod url_encoding {
    use url::form_urlencoded::Serializer;
    /// Parses the vector of key-value pairs into urlencoded payload serving as request body.
    ///
    /// # Arguments
    ///
    /// * `data` - Vector of key-value pairs
    ///    
    pub fn url_encode(data: &[(&str, &str)]) -> String {
        Serializer::new(String::new())
            .extend_pairs(data.iter())
            .finish()
    }
}

mod encryption {
    use hmac::{Hmac, Mac};
    use sha2::{Digest, Sha256, Sha512};
    type HmacSha512 = Hmac<Sha512>;
    /// Hashes the payload prefixed by nonce.
    ///
    /// # Arguments
    ///
    /// * `nonce` - A timestamp or value which increases per every request sent
    /// * `url_encoded_payload` - Data ready to be sent as request body
    ///    
    fn hash_payload(nonce: &str, url_encoded_payload: &str) -> Vec<u8> {
        let nonce_prepended_payload = [nonce, url_encoded_payload].concat().into_bytes();
        let hashed_payload = Sha256::new()
            .chain_update(nonce_prepended_payload)
            .finalize()
            .to_vec();
        hashed_payload
    }
    /// Creates a message consisting of hashed payload prefixed by endpoint path.
    ///
    /// # Arguments
    ///
    /// * `nonce` - A timestamp or value which increases per every request sent
    /// * `url_encoded_payload` - Data ready to be sent as request body
    /// * `endpoint_path` - Path to an endpoint, NOT prefixed by link to API
    ///
    fn build_message(nonce: &str, url_encoded_payload: &str, endpoint_path: &str) -> Vec<u8> {
        let hashed_payload = hash_payload(nonce, url_encoded_payload);
        let endpoint_path_bytes: Vec<u8> = endpoint_path.into();
        let message = [endpoint_path_bytes.as_slice(), hashed_payload.as_slice()].concat();
        message
    }

    fn get_mac(secret_bytes: &[u8], message: &[u8]) -> Vec<u8> {
        let mut mac =
            HmacSha512::new_from_slice(secret_bytes).expect("HMAC can take key of any size");
        mac.update(message.as_ref());
        mac.finalize().into_bytes().to_vec()
    }

    fn encrypt_message(message: &[u8], api_secret: &[u8]) -> String {
        let secret_bytes = base64::decode(api_secret).unwrap();
        let mac_bytes = get_mac(&secret_bytes, message);
        let signature = base64::encode(mac_bytes);
        signature
    }
    /// Creates signature used for authentication.
    ///
    /// # Arguments
    ///
    /// * `nonce` - A timestamp or value which increases per every request sent
    /// * `url_encoded_payload` - Data ready to be sent as request body
    /// * `endpoint_path` - Path to an endpoint, NOT prefixed by link to API
    /// * `api_secret` - Private key for API
    ///
    pub fn get_signature(
        nonce: &str,
        url_encoded_payload: &str,
        endpoint_path: &str,
        api_secret: &str,
    ) -> String {
        let message = build_message(nonce, url_encoded_payload, endpoint_path);
        let api_secret_bytes: Vec<u8> = api_secret.into();
        let signature = encrypt_message(&message, &api_secret_bytes);
        signature
    }
}

mod requesting {
    /// Sends GET request built from prepared payload and assembled signature
    ///
    /// # Arguments
    ///
    /// * `full_link` - Link to API combined with specified endpoint
    /// * `url_encoded_payload` - Data ready to be sent as request body
    /// * `api_key` - Public key to API
    /// * `signature` - Signature used for authentication
    ///
    pub async fn send_request(
        full_link: &str,
        url_encoded_payload: &str,
        api_key: &str,
        signature: &str,
    ) -> reqwest::Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let result = client
            .post(full_link)
            .body(url_encoded_payload.to_owned())
            .header("API-Key", api_key)
            .header("API-Sign", signature)
            .send()
            .await;
        result
    }
}
