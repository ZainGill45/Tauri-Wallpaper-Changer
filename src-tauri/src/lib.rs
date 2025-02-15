use winapi::um::winuser::{
    SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SPIF_SENDWININICHANGE,
};
use std::os::windows::ffi::OsStrExt;
use rand::seq::SliceRandom;
use platform_dirs::AppDirs;
use std::time::Duration;
use std::path::PathBuf;
use tauri::command;
use std::io::Write;
use base64::encode;
use std::thread;
use std::fs;

#[derive(serde::Deserialize, Debug)]
struct FileData {
    name: String,
    data: Vec<u8>,
}

#[derive(serde::Serialize, Debug)]
struct FileInfo {
    name: String,
    data: String,
}

fn file_to_base64(file_path: &PathBuf) -> Result<String, String> {
    let file_data = fs::read(file_path).map_err(|e| format!("Failed to read file {:?}: {}", file_path, e))?;
    Ok(encode(file_data))
}

/// - In development (debug builds), it uses the parent directory of current_dir() to reach "src/assets/images".
/// - In production, it uses Tauri’s app data directory (a writable location) and ensures an "images" subfolder exists.
fn get_images_dir() -> Result<PathBuf, String> {
    let app_dirs = AppDirs::new(Some("wallpaper-changer"), true)
        .ok_or("Failed to determine application directories")?;
    let images_dir = app_dirs.data_dir.join("images");

    if !images_dir.exists() {
        fs::create_dir_all(&images_dir)
            .map_err(|e| format!("Failed to create images directory: {}", e))?;
    }

    Ok(images_dir)
}

#[command]
fn get_files() -> Result<Vec<FileInfo>, String> {
    println!("Fetching list of images");
    let images_dir = get_images_dir()?;
    println!("Reading directory: {:?}", images_dir);
    let mut files = Vec::new();

    let dir_entries = fs::read_dir(&images_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in dir_entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        match file_to_base64(&path) {
                            Ok(encoded_data) => {
                                files.push(FileInfo {
                                    name: name.to_string(),
                                    data: encoded_data,
                                });
                                println!("Encoded file: {}", name);
                            }
                            Err(err) => {
                                println!("Error encoding file {}: {}", name, err);
                                continue;
                            }
                        }
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

#[command]
fn delete_image(file_name: String) -> Result<String, String> {
    println!("Attempting to delete image: {}", file_name);
    let images_dir = get_images_dir()?;
    println!("Images directory: {:?}", images_dir);
    let full_path = images_dir.join(&file_name);
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
fn delete_all_images() -> Result<String, String> {
    println!("Attempting to clear images directory");
    let images_dir = get_images_dir()?;
    println!("Clearing directory: {:?}", images_dir);

    let dir_entries = fs::read_dir(&images_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

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
    let images_dir = get_images_dir()?;
    println!("Images directory: {:?}", images_dir);
    
    fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;
    
    for file in files {
        println!("Processing file: {} (size: {} bytes)", file.name, file.data.len());
        let file_path = images_dir.join(&file.name);
        println!("Writing to path: {:?}", file_path);

        match fs::File::create(&file_path) {
            Ok(mut file_handle) => {
                file_handle.write_all(&file.data)
                    .map_err(|e| format!("Failed to write file '{}': {}", file.name, e))?;
                println!("Successfully wrote file: {}", file.name);
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

#[command]
fn set_random_wallpaper() -> Result<String, String> {
    let images_dir = get_images_dir()?;
    let entries = fs::read_dir(&images_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .map(|ext| ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .collect::<Vec<PathBuf>>();

    if entries.is_empty() {
        return Err("No images found in the upload directory".to_string());
    }

    let random_image = entries
        .choose(&mut rand::thread_rng())
        .ok_or("Failed to pick a random image")?;

    let path = random_image.to_str().ok_or("Invalid file path")?;
    let wide_path: Vec<u16> = std::ffi::OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let result = SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            wide_path.as_ptr() as *mut _,
            SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
        );
        if result == 0 {
            return Err(format!("Failed to set wallpaper for path: {:?}", random_image));
        }
    }

    Ok(format!("Successfully set wallpaper to: {:?}", random_image))
}

fn start_wallpaper_update() {
    loop {
        match set_random_wallpaper() {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("Error setting wallpaper: {}", e),
        }
        thread::sleep(Duration::from_secs(60));
    }
}

pub fn run() {
    println!("Starting Tauri application...");

    // Start the wallpaper update loop in a background thread
    std::thread::spawn(|| {
        start_wallpaper_update();
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            upload_files,
            delete_image,
            delete_all_images,
            get_files,
            set_random_wallpaper
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
