mod convert;

use convert::{BatchResult, ConvertOptions, ConvertSuccess, ImageInfo};

#[tauri::command]
fn probe_image(path: String) -> Result<ImageInfo, String> {
    convert::probe_image(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn convert_image(
    source_path: String,
    output_dir: String,
    options: ConvertOptions,
) -> Result<ConvertSuccess, String> {
    convert::convert_image(source_path, output_dir, options).map_err(|e| e.to_string())
}

#[tauri::command]
fn convert_batch(
    source_paths: Vec<String>,
    output_dir: String,
    options: ConvertOptions,
) -> Result<BatchResult, String> {
    Ok(convert::convert_batch(source_paths, output_dir, options))
}

#[tauri::command]
fn preview_image(path: String, max_edge: Option<u32>) -> Result<String, String> {
    convert::preview_data_url(path, max_edge.unwrap_or(480)).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            probe_image,
            convert_image,
            convert_batch,
            preview_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running Zayan Image Magic");
}
