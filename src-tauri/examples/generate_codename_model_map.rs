use std::path::PathBuf;

use tauri_app_lib::commands::rom_data::generate_codename_model_map_to_path;

#[tokio::main]
async fn main() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let output_path = root_dir
        .join("bin")
        .join("rom-data")
        .join("codename-model-map.json");

    match generate_codename_model_map_to_path(&output_path).await {
        Ok(result) => {
            println!("output_path={}", result.output_path);
            println!("total={}", result.total);
        }
        Err(error) => {
            eprintln!("error={}", error);
            std::process::exit(1);
        }
    }
}
