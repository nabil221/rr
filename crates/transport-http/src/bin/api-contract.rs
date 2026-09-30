use std::{env, error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let mode = env::args().nth(1).unwrap_or_else(|| "--check".to_owned());
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/src/generated/api-contract.ts");
    let generated = local_stack_proof_transport_http::typescript_contract();
    match mode.as_str() {
        "--write" => {
            fs::create_dir_all(
                path.parent()
                    .ok_or("Contract output directory is missing")?,
            )?;
            fs::write(&path, generated)?;
            println!("Generated {}", path.display());
        }
        "--check" => {
            let current = fs::read_to_string(&path)
                .unwrap_or_default()
                .replace("\r\n", "\n");
            if current != generated {
                return Err("HTTP contract drift detected; run npm run contract:generate and review the client changes".into());
            }
            println!("HTTP contract matches Rust DTOs");
        }
        _ => return Err("Usage: api-contract [--check | --write]".into()),
    }
    Ok(())
}
