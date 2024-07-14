use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

pub fn generate_proof(initial_state: i32, steps: &[i32]) -> Result<String, std::io::Error> {
    // Prepare input.json
    let input_data = serde_json::json!({
        "initial_state": initial_state,
        "steps": steps
    });

    let mut input_file = File::create("zk/input.json")
        .map_err(|e| eprintln!("We're here1"))
        .unwrap();
    write!(input_file, "{}", input_data)?;

    // Generate witness
    Command::new("node")
        .args(&[
            "zk/compute_js/generate_witness.js",
            "zk/compute_js/compute.wasm",
            "zk/input.json",
            "zk/witness.wtns",
        ])
        .output()
        .map_err(|e| eprintln!("We're here2"))
        .unwrap();

    // Generate the proof
    let out = Command::new("snarkjs")
        .args(&[
            "plonk",
            "prove",
            "zk/compute_final.zkey",
            "zk/witness.wtns",
            "zk/proof.json",
            "zk/public.json",
        ])
        .output()
        .map_err(|e| eprintln!("We're here3, {}", e))
        .unwrap();
    let formatted_proof = Command::new("snarkjs")
        .args(&[
            "zkey",
            "export",
            "soliditycalldata",
            "zk/public.json",
            "zk/proof.json",
        ])
        .output()
        .map_err(|e| eprintln!("We're here3, {}", e))
        .unwrap();

    println!("{:?}", formatted_proof);

    Ok(String::from_utf8_lossy(&formatted_proof.stdout).to_string())
}
