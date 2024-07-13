use aes::cipher::BlockSizeUser;
use clap::{Arg, Command, Parser};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use rand::rngs::OsRng;
use rocket::data::{Data, ToByteUnit};
use rocket::http::{ContentType, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::{post, routes, Config, Request, State};
use rsa::{
    pkcs8::DecodePublicKey, traits::PaddingScheme, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tfhe::integer::{BooleanBlock, RadixCiphertext, RadixClientKey};

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use base64::encode;
use csv::ReaderBuilder;
use rand::Rng;

type Aes256Cbc = cbc::Encryptor<aes::Aes256>;
use tfhe::prelude::*;
use tfhe::shortint::parameters::{PARAM_MESSAGE_2_CARRY_3_KS_PBS, PARAM_MESSAGE_2_CARRY_6_KS_PBS};
use tfhe::{
    generate_keys,
    integer::{gen_keys_radix, ServerKey},
    set_server_key, ConfigBuilder, FheUint64,
};
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Parser)]
pub struct StoreCmd {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,
}

struct ComputeTypeHeader(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ComputeTypeHeader {
    type Error = std::io::Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(header) = request.headers().get_one("compute_type") {
            Outcome::Success(ComputeTypeHeader(header.to_string()))
        } else {
            Outcome::Error((
                Status::BadRequest,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Missing compute_type header",
                ),
            ))
        }
    }
}

fn encrypt_file(file_path: PathBuf, key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut data = std::fs::read(file_path.clone()).expect("Unable to read file");
    let cipher = Aes256Cbc::new_from_slices(key, iv).unwrap();
    let mut buffer = vec![0u8; data.len() + Aes256::block_size()];
    buffer[..data.len()].copy_from_slice(&data);

    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len())
        .unwrap();

    let file_extension = file_path.extension().unwrap().to_str().unwrap();
    let metadata = format!("{}:{}", file_extension.len(), file_extension);

    [
        iv.to_vec(),
        metadata.as_bytes().to_vec(),
        ciphertext.to_vec(),
    ]
    .concat()
}

fn encrypt_symmetric_key(symmetric_key: &[u8], public_keys: &[RsaPublicKey]) -> Vec<u8> {
    public_keys[1]
        .encrypt(&mut OsRng, Pkcs1v15Encrypt, symmetric_key)
        .expect("failed to encrypt")
}

fn save_base64_to_file(filename: &str, data: &[u8]) {
    let base64_data = encode(data);
    let dir = Path::new(filename)
        .parent()
        .expect("No parent directory found");
    if !dir.exists() {
        fs::create_dir_all(dir).expect("Unable to create directory");
    }
    let mut file = File::create(filename).expect("Unable to create file");
    file.write_all(base64_data.as_bytes())
        .expect("Unable to write data");
}

fn read_csv_headers(file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;

    let headers = rdr.headers()?.clone();
    let headers_vec = headers.iter().map(String::from).collect::<Vec<_>>();

    Ok(headers_vec)
}

fn read_csv_column(file_path: &str, column: &str) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;

    let headers = rdr.headers()?.clone();
    let column_index = headers
        .iter()
        .position(|h| h == column)
        .ok_or("Column not found")?;

    let mut column_data = Vec::new();

    for result in rdr.records() {
        let record = result?;
        if let Some(value) = record.get(column_index) {
            if let Ok(parsed_value) = value.parse::<u64>() {
                column_data.push(parsed_value);
            } else {
                eprintln!("Warning: Skipping invalid value '{}'", value);
            }
        }
    }

    Ok(column_data)
}

struct KeyPair {
    client_key: RadixClientKey,
    server_key: ServerKey,
}
impl StoreCmd {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Data Processing, Input Path: {}", &self.input);
        let input_path = PathBuf::from(&self.input);
        let output_path = PathBuf::from(&self.output);
        let input_extension = input_path.extension().unwrap().to_str().unwrap();

        let enc_type = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Encryption Type")
            .items(&["FHE Encryption", "AES Symmetric Key Dual Encryption"])
            .interact()
            .unwrap();

        match enc_type {
            0 => {
                // check if csv, ask which row to encrypt,
                if input_extension != "csv" {
                    return Err("Only csv input allowed for FHE Enc".into());
                }
                let headers = read_csv_headers(input_path.to_str().unwrap())?;

                let selected_header = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(
                        "Select Header [Int Values] to Encrypt, Computation can only be performed on these",
                    )
                    .items(&headers)
                    .interact()
                    .unwrap();
                let selected_column = &headers[selected_header];
                let column_data = read_csv_column(input_path.to_str().unwrap(), &selected_column)?;
                log::info!("Encrypting data using Fully homomorphic encryption. Hold On Might Take a Minute!!");
                // generate client and server keys
                let (client_key, server_key) = gen_keys_radix(PARAM_MESSAGE_2_CARRY_3_KS_PBS, 8);
                let key_pair = Arc::new(KeyPair {
                    client_key: client_key.clone(),
                    server_key: server_key.clone(),
                });
                let mut serialized_data = Vec::new();
                bincode::serialize_into(&mut serialized_data, &server_key)?;
                for data in column_data {
                    let value = client_key.encrypt(data);
                    bincode::serialize_into(&mut serialized_data, &value);
                }
                save_base64_to_file(
                    format!(
                        "{}/{}/fhe_enc_data.b64",
                        output_path.to_str().unwrap(),
                        input_path.file_stem().unwrap().to_str().unwrap()
                    )
                    .as_str(),
                    &serialized_data,
                );

                #[post("/process_job", format = "application/octet-stream", data = "<data>")]
                async fn process_job<'a>(
                    content_type: &ContentType,
                    data: Data<'_>,
                    key_pair: &State<Arc<KeyPair>>,
                    compute_type: ComputeTypeHeader,
                ) -> Result<String, std::io::Error> {
                    let mut buffer = Vec::new();
                    data.open(400.mebibytes()).read_to_end(&mut buffer).await?;
                    let computetype = compute_type.0.as_str();

                    let output: String = match computetype {
                        "Average" | "Total" => {
                            let data: RadixCiphertext =
                                bincode::deserialize_from(&buffer[..]).unwrap();
                            let client_key = &key_pair.client_key;
                            let res: u64 = client_key.decrypt(&data);
                            format!("{}", res)
                        }
                        "GT" | "LT" | "GE" | "LE" => {
                            let data: BooleanBlock =
                                bincode::deserialize_from(&buffer[..]).unwrap();
                            let client_key = &key_pair.client_key;
                            let res: bool = client_key.decrypt_bool(&data);
                            format!("{}", res)
                        }

                        _ => ("Invalid Compute type").to_string(),
                    };
                    Ok(output)
                }

                let config = Config {
                    port: 3000,
                    ..Config::debug_default()
                };
                log::info!("Starting a client-side decrypt server on http://localhost:3000/ ");
                rocket::custom(&config)
                    .mount("/", routes![process_job])
                    .manage(key_pair)
                    .launch()
                    .await?;
            }
            1 => {
                log::info!("Encrypting data using Dual Aes encryption. Hold On Might Take a Minute!!");

                let symmetric_key: [u8; 32] = rand::thread_rng().gen();
                let iv: [u8; 16] = rand::thread_rng().gen();
                let encrypted_data = encrypt_file(input_path.clone(), &symmetric_key, &iv);
                save_base64_to_file(
                    format!(
                        "{}/{}/enc_data.b64",
                        output_path.to_str().unwrap(),
                        input_path.file_stem().unwrap().to_str().unwrap()
                    )
                    .as_str(),
                    &encrypted_data,
                );

                let pub_ke_filey = std::fs::read_to_string("./keys/key1/public_key1.pem").unwrap();
                let client_pub_key = RsaPublicKey::from_public_key_pem(&pub_ke_filey).unwrap();
                let response = reqwest::get("http://localhost:8000/pubkey")
                    .await?
                    .json::<serde_json::Value>()
                    .await?;
                let parsed_server_key_string = response
                    .get("pubkey")
                    .and_then(|value| Some(value.to_string()))
                    .unwrap();
                let parsed_server_key = parsed_server_key_string
                    .trim()
                    .replace("\\n", "\n")
                    .trim_matches('"')
                    .to_string();
                let server_pub_key = RsaPublicKey::from_public_key_pem(parsed_server_key.as_str())
                    .map_err(|err| format!("Failed to parse server public key: {}", err))?;
                let enc_sym_key = encrypt_symmetric_key(
                    &symmetric_key,
                    &vec![client_pub_key.clone(), server_pub_key],
                );
                save_base64_to_file(
                    format!(
                        "{}/{}/enc_sym_keys.b64",
                        output_path.to_str().unwrap(),
                        input_path.file_stem().unwrap().to_str().unwrap()
                    )
                    .as_str(),
                    &enc_sym_key,
                );
                log::info!("Data successfully Processed!!");
            }
            _ => {
                eprintln!("Invalid Input");
                return Err("Invalid Input".into());
            }
        }

        Ok(())
    }
}
