use std::fs;
use std::io::Write;
use tauri::command;

#[derive(serde::Deserialize)]
struct FileData {
    name: String,
    data: Vec<u8>,
}

#[command]
fn upload_files(files: Vec<FileData>) -> Result<String, String> {
    let upload_dir = "./uploads"; // Local directory to save files

    // Ensure the upload directory exists
    if let Err(err) = fs::create_dir_all(upload_dir) {
        return Err(format!("Failed to create upload directory: {}", err));
    }

    for file in files {
        let file_path = format!("{}/{}", upload_dir, file.name);
        let mut file_handle = match fs::File::create(&file_path) {
            Ok(file) => file,
            Err(err) => return Err(format!("Failed to create file '{}': {}", file.name, err)),
        };

        if let Err(err) = file_handle.write_all(&file.data) {
            return Err(format!("Failed to write to file '{}': {}", file.name, err));
        }
    }

    Ok("Files uploaded successfully!".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![upload_files])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
