use tauri::command;
use std::io::Write;
use std::fs;

#[derive(serde::Deserialize, Debug)]
struct FileData {
    name: String,
    data: Vec<u8>,
}

#[derive(serde::Serialize, Debug)]
struct FileInfo {
    name: String,
    path: String,
}

#[command]
fn get_files() -> Result<Vec<FileInfo>, String> {
    println!("Fetching list of images");
    
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
    
    println!("Reading directory: {:?}", images_dir);

    // Read the directory and collect file information
    let mut files = Vec::new();
    
    let dir_entries = fs::read_dir(&images_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in dir_entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Create relative path from src/assets/images
                        let relative_path = format!("/src/assets/images/{}", name);
                        files.push(FileInfo {
                            name: name.to_string(),
                            path: relative_path,
                        });
                        println!("Found file: {}", name);
                    }
                }
            },
            Err(e) => {
                println!("Error reading entry: {}", e);
                continue;
            }
        }
    }

    println!("Found {} files", files.len());
    Ok(files)
}

#[tauri::command]
fn delete_image(file_path: String) -> Result<String, String> {
    println!("Attempting to delete image with path: {}", file_path);
    
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    let upload_dir = current_dir
        .parent()
        .ok_or("Could not find parent directory")?
        .join("src")
        .join("assets")
        .join("images");

    println!("Upload directory: {:?}", upload_dir);

    let full_path = upload_dir.join(file_path);

    println!("Full file path to delete: {:?}", full_path);

    if full_path.is_file() {
        fs::remove_file(&full_path)
            .map_err(|e| format!("Failed to delete file {:?}: {}", full_path, e))?;

        println!("Successfully deleted: {:?}", full_path);
        Ok(format!("Successfully deleted file: {:?}", full_path))
    } else {
        let err_msg = format!("File not found: {:?}", full_path);
        println!("{}", err_msg);
        Err(err_msg)
    }
}

#[command]
fn upload_files(files: Vec<FileData>) -> Result<String, String> {
    println!("Received {} files for upload", files.len());
    
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    let upload_dir = current_dir
        .parent()
        .ok_or("Could not find parent directory")?
        .join("src")
        .join("assets")
        .join("images");
    
    println!("Upload directory: {:?}", upload_dir);
    
    match fs::create_dir_all(&upload_dir) {
        Ok(_) => println!("Upload directory created/verified successfully"),
        Err(e) => {
            let err_msg = format!("Failed to create upload directory: {}", e);
            println!("{}", err_msg);
            return Err(err_msg);
        }
    }
    
    for file in files {
        println!("Processing file: {} (size: {} bytes)", file.name, file.data.len());
        
        let file_path = upload_dir.join(&file.name);
        println!("Writing to path: {:?}", file_path);
        
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
        .invoke_handler(tauri::generate_handler![upload_files, delete_image, get_files])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}