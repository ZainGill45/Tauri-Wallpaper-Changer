use std::fs;
use std::io::Write;
use tauri::command;
use std::path::PathBuf;

#[derive(serde::Deserialize, Debug)]
struct FileData {
    name: String,
    data: Vec<u8>,
}

#[command]
fn clear_images() -> Result<String, String> {
    println!("Attempting to clear images directory");
    
    // Get the current executable's directory
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    // Navigate to the images directory
    let images_dir = current_dir
        .parent()
        .ok_or("Could not find parent directory")?
        .join("src")
        .join("assets")
        .join("images");
    
    println!("Clearing directory: {:?}", images_dir);

    // Read the directory
    let dir_entries = fs::read_dir(&images_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    // Delete each file
    for entry in dir_entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    match fs::remove_file(&path) {
                        Ok(_) => println!("Successfully deleted: {:?}", path),
                        Err(e) => {
                            let err_msg = format!("Failed to delete file {:?}: {}", path, e);
                            println!("{}", err_msg);
                            return Err(err_msg);
                        }
                    }
                }
            },
            Err(e) => {
                let err_msg = format!("Failed to read directory entry: {}", e);
                println!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }

    println!("Successfully cleared all files from images directory");
    Ok("Images directory cleared successfully!".to_string())
}

#[command]
fn upload_files(files: Vec<FileData>) -> Result<String, String> {
    println!("Received {} files for upload", files.len());
    
    // Get the current executable's directory
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    // Navigate to the project root and then to assets/images
    let upload_dir = current_dir
        .parent()
        .ok_or("Could not find parent directory")?
        .join("src")
        .join("assets")
        .join("images");
    
    println!("Upload directory: {:?}", upload_dir);
    
    // Create the upload directory if it doesn't exist
    match fs::create_dir_all(&upload_dir) {
        Ok(_) => println!("Upload directory created/verified successfully"),
        Err(e) => {
            let err_msg = format!("Failed to create upload directory: {}", e);
            println!("{}", err_msg);
            return Err(err_msg);
        }
    }
    
    // Process each file
    for file in files {
        println!("Processing file: {} (size: {} bytes)", file.name, file.data.len());
        
        let file_path = upload_dir.join(&file.name);
        println!("Writing to path: {:?}", file_path);
        
        // Create and write the file
        match fs::File::create(&file_path) {
            Ok(mut file_handle) => {
                match file_handle.write_all(&file.data) {
                    Ok(_) => println!("Successfully wrote file: {}", file.name),
                    Err(e) => {
                        let err_msg = format!("Failed to write file '{}': {}", file.name, e);
                        println!("{}", err_msg);
                        return Err(err_msg);
                    }
                }
            },
            Err(e) => {
                let err_msg = format!("Failed to create file '{}': {}", file.name, e);
                println!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    println!("All files processed successfully");
    Ok("Files uploaded successfully!".to_string())
}

pub fn run() {
    println!("Starting Tauri application...");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![upload_files, clear_images])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}