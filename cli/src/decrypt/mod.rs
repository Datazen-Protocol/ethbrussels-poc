use aes::cipher::BlockSizeUser;
use rocket::fs::NamedFile;
use rsa::{pkcs8::DecodePrivateKey, Pkcs1v15Encrypt, RsaPrivateKey};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use aes::Aes256;
use base64::{decode as base64_decode, encode as base64_encode};

fn decrypt_file(encrypted_data: &[u8], key: &[u8]) -> (Vec<u8>, String) {
    let iv_size = Aes256::block_size();
    let (iv, rest) = encrypted_data.split_at(iv_size);

    let metadata_len_end = rest.iter().position(|&b| b == b':').unwrap();
    let metadata_len: usize = std::str::from_utf8(&rest[..metadata_len_end])
        .unwrap()
        .parse()
        .unwrap();
    let metadata_end = metadata_len_end + 1 + metadata_len;

    let file_extension = std::str::from_utf8(&rest[metadata_len_end + 1..metadata_end])
        .unwrap()
        .to_string();
    let encrypted_data = &rest[metadata_end..];

    let cipher = cbc::Decryptor::<Aes256>::new_from_slices(key, iv).unwrap();
    let mut buffer = encrypted_data.to_vec();
    let decrypted_data = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer).unwrap();
    (decrypted_data.to_vec(), file_extension)
}

fn decrypt_symmetric_key(encrypted_key: &[u8], private_key: &RsaPrivateKey) -> Vec<u8> {
    private_key
        .decrypt(Pkcs1v15Encrypt, encrypted_key)
        .expect("failed to decrypt")
}

fn save_to_file(filename: &str, data: &[u8]) {
    let mut file = File::create(filename).expect("unable to create file");
    file.write_all(data).expect("unable to write data");
}

pub async fn decrypt(
    encrypted_file: String,
    encrypted_key_file: String,
    private_key_file: String,
    output_dir: String,
) -> Result<NamedFile, String> {
    let encrypted_file_path = PathBuf::from(encrypted_file);
    let encrypted_key_file_path = PathBuf::from(encrypted_key_file);
    let private_key_file_path = PathBuf::from(private_key_file);
    let output_dir_path = PathBuf::from(output_dir);

    // Decode base64 encoded encrypted file content
    let encrypted_data_base64 =
        std::fs::read_to_string(encrypted_file_path).expect("Unable to read encrypted file");
    let encrypted_data =
        base64_decode(&encrypted_data_base64).expect("Failed to decode base64 encrypted file");

    // Read and decode base64 encrypted keys
    let encrypted_keys_base64 = std::fs::read_to_string(encrypted_key_file_path)
        .expect("Unable to read encrypted key file");
    let encrypted_keys =
        base64_decode(&encrypted_keys_base64).expect("Failed to decode base64 encrypted key");

    // Read private key
    let private_key_pem =
        std::fs::read_to_string(private_key_file_path).expect("Unable to read private key file");
    let private_key =
        RsaPrivateKey::from_pkcs8_pem(&private_key_pem).expect("Unable to parse private key");

    // decrypt symm key
    let symmetric_key_server_half = decrypt_symmetric_key(&encrypted_keys, &private_key);

    // Decrypt the file using the decrypted symmetric key half
    let (decrypted_data, file_extension) =
        decrypt_file(&encrypted_data, &symmetric_key_server_half);

    // Create temporary directory for decrypted file
    let temp_dir = tempdir().map_err(|err| err.to_string())?;
    let temp_file_path = temp_dir
        .path()
        .join(format!("decrypted.{}", file_extension));
    let mut temp_file = match File::create(&temp_file_path) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };
    temp_file
        .write_all(&decrypted_data)
        .map_err(|err| err.to_string())?;

    // Return decrypted file as a named file for streaming
    let named_file = NamedFile::open(temp_file_path)
        .await
        .map_err(|err| err.to_string())?;
    Ok(named_file)
}
