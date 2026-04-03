use modell_converter::{convert::convert, format::Format};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![convert_model, write_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
