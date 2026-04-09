use modell_converter::{convert::{convert, batch}, format::Format};

/// Convert a model file from one format to another.
#[tauri::command]
fn convert_model(
    input_bytes: Vec<u8>,
    from: Format,
    to: Format,
) -> Result<Vec<u8>, String> {
    convert(&input_bytes, &from, &to)
        .map_err(|e| e.to_string())
}

/// Write bytes to a file at the specified path.
#[tauri::command]
fn write_file(path: String, bytes: Vec<u8>) -> Result<(), String> {
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct BatchResult {
    converted: usize,
    errors: usize,
}

/// Convert all matching files in a folder or zip archive.
#[tauri::command]
fn convert_batch(
    input_path: String,
    output_path: String,
    from: Format,
    to: Format,
) -> Result<BatchResult, String> {
    let (converted, errors) = batch(from, to, input_path.as_ref(), output_path.as_ref())
        .map_err(|e| e.to_string())?;
    Ok(BatchResult { converted, errors })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![convert_model, write_file, convert_batch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
