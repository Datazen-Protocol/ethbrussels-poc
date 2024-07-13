use clap::Parser;
use rand::rngs::OsRng;
use rsa::{
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
    RsaPrivateKey, RsaPublicKey,
};
use std::fs::{self, File};
use std::io::Write;

#[derive(Debug, Clone, Parser)]
pub struct KeygenCmd;

impl KeygenCmd {
    pub async fn execute(&self) -> Result<(), String> {
        gen_and_save_keys();
        Ok(())
    }
}

fn generate_rsa_keys() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = OsRng;
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    (private_key, public_key)
}

fn save_key_to_file(filename: &str, key: &[u8]) {
    let mut file = File::create(filename).expect("unable to create file");
    file.write_all(key).expect("unable to write data");
}

fn gen_and_save_keys() {
    fs::create_dir_all("keys/").expect("unable to create directory");
    let (private_key1, public_key1) = generate_rsa_keys();

    save_key_to_file(
        "keys/private_key.pem",
        &private_key1
            .to_pkcs8_pem(LineEnding::default())
            .unwrap()
            .as_bytes(),
    );
    save_key_to_file(
        "keys/public_key.pem",
        &public_key1
            .to_public_key_pem(LineEnding::default())
            .unwrap()
            .as_bytes(),
    );
}
