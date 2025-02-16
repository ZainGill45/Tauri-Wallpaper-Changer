use base64::encode;
use image::imageops::FilterType;
use platform_dirs::AppDirs;
use rand::seq::SliceRandom;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tauri::{command, Manager};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use winapi::um::winuser::{
    SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
};

const MAX_IMAGE_SIZE: u32 = 4096;
const THUMBNAIL_SIZE: u32 = 360;
const MAX_BATCH_SIZE: usize = 8;

#[derive(serde::Deserialize, Debug)]
struct FileData {
    name: String,
    data: Vec<u8>,
}

#[derive(serde::Serialize, Debug)]
struct FileInfo {
    name: String,
    data: String,
    thumbnail: String,
}

fn process_files_in_chunks<T, F>(items: Vec<T>, chunk_size: usize, mut f: F) -> Result<(), String>
where
    F: FnMut(&[T]) -> Result<(), String>,
{
    for chunk in items.chunks(chunk_size) {
        f(chunk)?;
    }
    Ok(())
}

fn optimize_image(data: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(data).map_err(|e| format!("Failed to load image: {}", e))?;

    let img = if img.width() > MAX_IMAGE_SIZE || img.height() > MAX_IMAGE_SIZE {
        let ratio = MAX_IMAGE_SIZE as f32 / img.width().max(img.height()) as f32;
        let new_width = (img.width() as f32 * ratio) as u32;
        let new_height = (img.height() as f32 * ratio) as u32;
        img.resize(new_width, new_height, FilterType::Lanczos3)
    } else {
        img
    };

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    img.write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode image: {}", e))?;

    Ok(buffer)
}

fn create_thumbnail(data: &[u8]) -> Result<String, String> {
    let img =
        image::load_from_memory(data).map_err(|e| format!("Failed to create thumbnail: {}", e))?;

    let thumbnail = img.resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Lanczos3);

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    thumbnail
        .write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;

    Ok(encode(&buffer))
}

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
async fn upload_files(files: Vec<FileData>) -> Result<String, String> {
    println!("Received {} files for upload", files.len());
    let images_dir = get_images_dir()?;

    process_files_in_chunks(files, MAX_BATCH_SIZE, |chunk| {
        for file in chunk {
            let optimized_data = optimize_image(&file.data)?;
            let file_path = images_dir.join(&file.name);

            fs::write(&file_path, &optimized_data)
                .map_err(|e| format!("Failed to write file '{}': {}", file.name, e))?;
        }
        Ok(())
    })?;

    Ok("Files uploaded successfully!".to_string())
}

#[command]
fn count_files() -> Result<usize, String> {
    let images_dir = get_images_dir()?;

    let file_count = fs::read_dir(&images_dir)
        .map_err(|e| format!("Failed to read images directory: {}", e))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .count();

    Ok(file_count)
}

#[command]
fn open_images_directory() -> Result<String, String> {
    let images_dir = get_images_dir()?;
    open::that(&images_dir).map_err(|e| format!("Failed to open images directory: {}", e))?;
    Ok(format!("Opened images directory: {:?}", images_dir))
}

#[command]
async fn get_files() -> Result<Vec<FileInfo>, String> {
    let images_dir = get_images_dir()?;
    let mut files = Vec::new();

    let entries: Vec<_> = fs::read_dir(&images_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(Result::ok)
        .collect();

    process_files_in_chunks(entries, MAX_BATCH_SIZE, |chunk| {
        for entry in chunk {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Ok(data) = fs::read(&path) {
                    if let Ok(thumbnail) = create_thumbnail(&data) {
                        files.push(FileInfo {
                            name: name.to_string(),
                            data: encode(&data),
                            thumbnail,
                        });
                    }
                }
            }
        }
        Ok(())
    })?;

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

    let dir_entries =
        fs::read_dir(&images_dir).map_err(|e| format!("Failed to read directory: {}", e))?;

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
            }
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
            return Err(format!(
                "Failed to set wallpaper for path: {:?}",
                random_image
            ));
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
        thread::sleep(Duration::from_secs(600));
    }
}

pub fn run() {
    println!("Starting Tauri application...");

    std::thread::spawn(|| {
        start_wallpaper_update();
    });

    tauri::Builder::default()
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    "show" => {
                        println!("show menu item was clicked");
                        // Show the window when the tray icon is clicked
                        app.get_webview_window("main").unwrap().show().unwrap();
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                println!("Hiding window and minizing to tray");
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            upload_files,
            delete_image,
            delete_all_images,
            count_files,
            open_images_directory,
            get_files,
            set_random_wallpaper
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
