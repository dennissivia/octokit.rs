use super::OctokitError;
use openssl::hash::MessageDigest;
use openssl::memcmp;
use openssl::sign::Signer;

pub const EVENT_HEADER_NAME: &str = "X-GITHUB-EVENT";
pub const SIGNATURE_HEADER_NAME: &str = "X-HUB-SIGNATURE";

pub fn verify_payload_signature(
    signature: &Option<String>,
    secret: &String,
    body: &String,
) -> bool {
    match signature {
        None => false,
        Some(sig) => {
            let result = verify(&sig, &secret, &body);
            match result {
                Ok(validity) => {
                    println!("signature verification resulted in: {}", validity);
                    validity
                }
                Err(err) => {
                    println!("payload verification failed with: {}", err);
                    false
                }
            }
        }
    }
}

///  Example signature header
///  "x-hub-signature": "sha1=4b4a1c9a70dc40caf22099fb2d62a283dedd4614"
fn verify(signature: &String, secret: &String, body: &String) -> Result<bool, OctokitError> {
    let secret = secret.as_bytes();
    let body = body.as_bytes();

    // discard the 'sha1='-prefix
    let sighex = &signature[5..];
    // decode sha1 has hex bytes
    let sigbytes = hex::decode(sighex)?;
    // Create a PKey
    let key = openssl::pkey::PKey::hmac(secret)?;
    // Compute the HMAC
    let mut signer = Signer::new(MessageDigest::sha1(), &key)?;
    signer.update(body)?;
    let hmac = signer.sign_to_vec()?;

    Ok(memcmp::eq(&hmac, &sigbytes))
}
