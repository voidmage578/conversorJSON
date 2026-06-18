// Declara o módulo file_reader.rs que criámos — isto diz ao Rust
// "existe um ficheiro file_reader.rs, inclui-o na compilação".
mod file_reader;
mod validation;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // invoke_handler regista quais funções Rust ficam acessíveis ao frontend.
        // generate_handler! é uma macro que junta a lista de comandos.
        .invoke_handler(tauri::generate_handler![
            file_reader::read_file_preview,
            validation::validate_file,
            validation::export_to_json
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
