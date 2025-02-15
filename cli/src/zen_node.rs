use crate::decrypt::decrypt;
use crate::lighthouse::upload_file;
use crate::zk_proof::generate_proof;
use base64::decode;
use clap::Parser;
use lazy_static::lazy_static;
use rocket::data::ToByteUnit;
use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::serde::json::json;
use rocket::{get, post, routes, Data};
use rocket_cors;
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tfhe::integer::{RadixCiphertext, ServerKey};

#[derive(Serialize)]
struct PubkeyResult {
    pubkey: String,
}

const DEFAULT_KEY_PATH: &str = "/keys";

lazy_static! {
    static ref KEY_PATH: Mutex<String> = Mutex::new(String::new());
    static ref USER_DATA: Mutex<HashMap<String, Vec<UserState>>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Parser)]
pub struct ZenNodeCmd {
    #[arg(short, long, default_value = DEFAULT_KEY_PATH)]
    key_file: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct UserState {
    key: String,
    description: String,
    file_id: String,
}

impl ZenNodeCmd {
    pub async fn execute(&self) -> Result<(), String> {
        if !self.key_file.exists() {
            log::error!(
                "Key Path Not Found 🧐 Please cross check: Path: {:?}",
                &self.key_file
            );
            return Err("Key path not found".to_string());
        }
        let key_path = PathBuf::from(&self.key_file);
        {
            let mut key_path_lock = KEY_PATH.lock().unwrap();
            *key_path_lock = self.key_file.to_str().unwrap().to_string();
        }
        if key_path.extension().unwrap_or_default() != "pem" {
            log::error!("Just .pem files allowed!! Use key-gen command to generate one");
            return Err("Invalid key path, should be a .pem file".to_string());
        }
        // Create store directory
        let _ = std::fs::create_dir_all("store/");
        log::info!(
            "✨Zen-node✨ Started on http://localhost:8000/ \n You're ready to store and compute"
        );
        // ...

        let cors = rocket_cors::CorsOptions::default()
            .allow_credentials(true)
            .to_cors()
            .unwrap();

        let _rocket = rocket::build()
            .mount(
                "/",
                routes![
                    store_handler,
                    pubkey_handler,
                    userdata_handler,
                    alldata_handler,
                    compute_handler
                ],
            )
            .attach(cors)
            .launch()
            .await;
        Ok(())
    }
}

#[post("/store", data = "<data>")]
async fn store_handler(
    content_type: &ContentType,
    data: Data<'_>,
) -> Result<String, std::io::Error> {
    log::info!("🚛 🚛 Data Coming In !!");
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("data").size_limit(u64::from(1000.mebibytes())),
        MultipartFormDataField::file("enc_symm_key").size_limit(u64::from(32.mebibytes())),
        MultipartFormDataField::text("address"),
        MultipartFormDataField::text("filename"),
        MultipartFormDataField::text("description"),
    ]);

    // Attempt to parse the form data
    let multi_form_data = match MultipartFormData::parse(content_type, data, options).await {
        Ok(data) => data,
        Err(err) => {
            log::error!("Data Store Failed 😭. Error: {}", err);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Failed to parse multipart form data",
            ));
        }
    };

    let data_file = multi_form_data.files.get("data");
    let enc_symm_key_file = multi_form_data.files.get("enc_symm_key");
    let address = multi_form_data.texts.get("address").unwrap();
    let filename = multi_form_data.texts.get("filename").unwrap();
    let description = multi_form_data.texts.get("description").unwrap();

    let address = &address[0].text;
    let filename = &filename[0].text;
    let description = &description[0].text;

    // Limit the scope of the MutexGuard
    let mut data_path_final: String = String::new();
    {
        let mut user_state = USER_DATA.lock().unwrap();

        // Create the directory for the files
        let file_path = format!("store/{}/{}", address, filename);
        let _ = std::fs::create_dir_all(&file_path);

        if let Some(data_file_fields) = data_file {
            let data_file_field = &data_file_fields[0];
            let data_file_name = data_file_field.file_name.as_ref().unwrap();
            let final_path = format!("{}/{}", file_path, data_file_name);
            data_path_final = final_path.clone();
            let mut file = File::create(final_path)?;
            let mut temp_file = File::open(&data_file_field.path)?;
            let mut buffer: Vec<u8> = Vec::new();
            temp_file.read_to_end(&mut buffer)?;
            file.write_all(&buffer)?;

            // Update user state with data file
            let user_entry = user_state
                .entry(address.clone())
                .or_insert(Vec::<UserState>::new());
            user_entry.push(UserState {
                key: String::new(),
                description: description.clone(),
                file_id: filename.clone(),
            });
        } else {
            log::error!("Data Store Failed 😭. Error: Data file not found");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Data file not found",
            ));
        }
    }

    if let Some(enc_symm_key_file_fields) = enc_symm_key_file {
        ()
    } else {
        log::warn!("Encrypted symmetric key file not found in the form data");
    }

    // Make the asynchronous call outside the lock scope
    let _lh_resp: crate::lighthouse::LighthouseResponse =
        upload_file(&data_path_final).await.unwrap();
    Ok(format!("{:?}", _lh_resp))
}

#[get("/pubkey")]
async fn pubkey_handler() -> Result<String, std::io::Error> {
    println!("Pub Key Requested");
    let pubkey_path = KEY_PATH.lock().unwrap().clone();
    let pubkey = tokio::fs::read_to_string(pubkey_path)
        .await
        .map_err(|_| {
            eprintln!("Failed to read public key file");
            rocket::http::Status::InternalServerError
        })
        .unwrap();
    let res = PubkeyResult { pubkey };
    Ok(serde_json::to_string(&res).unwrap())
}

#[get("/userdata/<address>")]
async fn userdata_handler(address: String) -> Result<String, std::io::Error> {
    println!("User Data Requested for {}", address);
    let mut user_state = USER_DATA.lock().unwrap();
    if let Some(data) = user_state.get(&address) {
        Ok(serde_json::to_string(data).unwrap())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "User Data not found",
        ))
    }
}

#[get("/alldata")]
async fn alldata_handler() -> Result<String, std::io::Error> {
    println!("All user data requested");
    let mut user_state = USER_DATA.lock().unwrap();
    Ok(serde_json::to_string(&*user_state).unwrap())
}

/*  compute on fhe -> we need file path [already done] -> Type of compute [average ( int res ), total (int res),
    gt, lt, gt_eq, lt_eq (bool res)]
*/

#[derive(Deserialize)]
enum ComputeTypes {
    Average,
    Total,
    GT,
    LT,
    GE,
    LE,
}

impl Display for ComputeTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputeTypes::Average => write!(f, "Average"),
            ComputeTypes::Total => write!(f, "Total"),
            ComputeTypes::GT => write!(f, "GT"),
            ComputeTypes::LT => write!(f, "LT"),
            ComputeTypes::GE => write!(f, "GE"),
            ComputeTypes::LE => write!(f, "LE"),
        }
    }
}

#[derive(Deserialize)]
struct ComputeInput {
    address: String,
    filename: String,
    compute_type: ComputeTypes,
    threshold: Option<u32>,
}

#[post("/compute", data = "<input>")]
async fn compute_handler(
    input: rocket::serde::json::Json<ComputeInput>,
) -> Result<String, io::Error> {
    // Paths
    let initial_state = 1;
    let mut steps: Vec<i32> = vec![];
    let data_dir: String = format!("store/{}/{}", &input.address, &input.filename);
    steps.push(2);
    if input.filename.starts_with("fhe") {
        let mut file = File::open(format!("{}/fhe_enc_data.b64", data_dir))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let decoded_data = decode(&data).unwrap();
        let serialized_data_bytes = decoded_data.as_slice();
        let mut serialized_data = Cursor::new(serialized_data_bytes);
        let server_key: ServerKey = bincode::deserialize_from(&mut serialized_data).unwrap();
        let mut values: Vec<RadixCiphertext> = vec![];
        while true {
            let int: RadixCiphertext = match bincode::deserialize_from(&mut serialized_data) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("Failed to deserialize data: {:?}", err);
                    break;
                }
            };
            values.push(int);
        }
        let mut serial_res: Vec<u8> = Vec::new();
        steps.push(3);
        steps.push(4);
        let compute_result: Result<String, _> = match input.compute_type {
            ComputeTypes::Average => {
                let mut sum: RadixCiphertext =
                    ServerKey::sum_ciphertexts_parallelized(&server_key, &values).unwrap();
                let count = values.len() as u32;
                let average =
                    ServerKey::scalar_div_assign_parallelized(&server_key, &mut sum, count);
                let _ = bincode::serialize_into(&mut serial_res, &average);
                let res = get_decoded_res(ComputeTypes::Average, serial_res)
                    .await
                    .unwrap();
                Ok::<String, Box<dyn std::error::Error>>(res)
            }
            ComputeTypes::Total => {
                let sum: RadixCiphertext =
                    ServerKey::sum_ciphertexts_parallelized(&server_key, &values).unwrap();
                let _ = bincode::serialize_into(&mut serial_res, &sum);
                let res = get_decoded_res(ComputeTypes::Total, serial_res)
                    .await
                    .unwrap();
                Ok(res)
            }
            ComputeTypes::GT => {
                let threshold: u32 = input.threshold.unwrap();
                let sum: RadixCiphertext =
                    ServerKey::unchecked_sum_ciphertexts_vec_parallelized(&server_key, values)
                        .unwrap();
                let is_gt = ServerKey::scalar_gt_parallelized(&server_key, &sum, threshold);
                let _ = bincode::serialize_into(&mut serial_res, &is_gt);
                let res = get_decoded_res(ComputeTypes::GT, serial_res).await.unwrap();
                Ok(res)
            }
            ComputeTypes::LT => {
                let threshold: u32 = input.threshold.unwrap();
                let sum: RadixCiphertext =
                    ServerKey::unchecked_sum_ciphertexts_vec_parallelized(&server_key, values)
                        .unwrap();
                let is_gt = ServerKey::scalar_lt_parallelized(&server_key, &sum, threshold);
                let _ = bincode::serialize_into(&mut serial_res, &is_gt);
                let res = get_decoded_res(ComputeTypes::LT, serial_res).await.unwrap();
                Ok(res)
            }
            ComputeTypes::GE => {
                let threshold: u32 = input.threshold.unwrap();
                let sum: RadixCiphertext =
                    ServerKey::unchecked_sum_ciphertexts_vec_parallelized(&server_key, values)
                        .unwrap();
                let is_gt = ServerKey::scalar_ge_parallelized(&server_key, &sum, threshold);
                let _ = bincode::serialize_into(&mut serial_res, &is_gt);
                let res = get_decoded_res(ComputeTypes::GE, serial_res).await.unwrap();
                Ok(res)
            }
            ComputeTypes::LE => {
                let threshold: u32 = input.threshold.unwrap();
                let sum: RadixCiphertext =
                    ServerKey::unchecked_sum_ciphertexts_vec_parallelized(&server_key, values)
                        .unwrap();
                let is_gt = ServerKey::scalar_le_parallelized(&server_key, &sum, threshold);
                let _ = bincode::serialize_into(&mut serial_res, &is_gt);
                let res = get_decoded_res(ComputeTypes::GT, serial_res).await.unwrap();
                Ok(res)
            }
        };
        steps.push(5);
        // send proof to chain, return result, proof and tx hash
        let proof: String = generate_proof(initial_state, &steps).unwrap();
        println!("{}", compute_result.as_ref().unwrap());
        let response_json = json!({
            "compute_result": compute_result.unwrap(),
            "proof": proof
        });
        Ok(response_json.to_string())
    } else {
        let data_file_path = format!("{}/enc_data.b64", data_dir);
        let enc_symm_key_path = format!("{}/enc_sym_keys.b64", data_dir);
        Ok(("Compute Done".to_string()))
    }
}

async fn get_decoded_res(
    compute_type: ComputeTypes,
    serial_enc_output: Vec<u8>,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = reqwest::Client::new()
        .post("http://localhost:6000/process_job")
        .header("Content-Type", "application/octet-stream")
        .header("compute_type", compute_type.to_string())
        .body(serial_enc_output)
        .send()
        .await
        .map_err(|err| {
            eprintln!("Failed to send data to server: {:?}", err);
            io::Error::new(io::ErrorKind::Other, "Failed to send data to server")
        })?;
    let res = output
        .text()
        .await
        .map_err(|err| {
            eprintln!("Failed to get text response, {}", err);
            io::Error::new(io::ErrorKind::Other, "Failed to get text response")
        })
        .unwrap();
    Ok(res)
}
