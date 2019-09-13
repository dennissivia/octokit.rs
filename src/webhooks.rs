use openssl::hash::MessageDigest;
use openssl::memcmp;
use openssl::sign::Signer;

pub const EVENT_HEADER_NAME: &str = "X-GITHUB-EVENT";
pub const SIGNATURE_HEADER_NAME: &str = "X-HUB-SIGNATURE";

///  Example signature header
///  "x-hub-signature": "sha1=4b4a1c9a70dc40caf22099fb2d62a283dedd4614"
pub fn verify_payload_signature(
    signature: &Option<String>,
    secret: &String,
    body: &String,
) -> bool {
    let secret = secret.as_bytes();
    let body = body.as_bytes();

    match signature {
        Some(sig) => {
            // discard the 'sha1='-prefix
            let sighex = &sig[5..];
            // decode sha1 has hex bytes
            let sigbytes = hex::decode(sighex).expect("Decoding failed");

            // Create a PKey
            let key = openssl::pkey::PKey::hmac(secret).unwrap();

            // Compute the HMAC
            let mut signer = Signer::new(MessageDigest::sha1(), &key).unwrap();
            signer.update(body).unwrap();
            let hmac = signer.sign_to_vec().unwrap();

            println!("signature: sha1={:?}", sighex);
            println!("bytes: {:?}", sigbytes);
            println!("hmac is: {:?}", hmac);
            //            println!("hmac len: {}, sig len: {}", hmac.len(), sigbytes.len());

            let valid = memcmp::eq(&hmac, &sigbytes);
            println!("validity is: {:?}", valid);
            valid
        }
        None => false,
    }
}
