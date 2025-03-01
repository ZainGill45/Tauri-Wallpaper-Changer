use actix_files as fs;
use actix_web::{App, HttpServer};
use platform_dirs::AppDirs;
use rand::seq::SliceRandom;
use std::fs as std_fs;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::sync::{Condvar, Arc, Mutex};
use serde_json::json;
use tauri::{command, Manager};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_store::StoreExt;
use tokio::fs as tokio_fs;
use winapi::um::winuser::{
    SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
};

lazy_static::lazy_static! {
    static ref GLOBAL_WALLPAPER_CHANGE_INTERVAL: Arc<Mutex<u64>> = Arc::new(Mutex::new(300));
    static ref GLOBAL_CONDVAR: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));
}

#[command]
fn modify_wallpaper_change_interval(new_change_interval: u64, app_handle: tauri::AppHandle) {
    let mut interval = GLOBAL_WALLPAPER_CHANGE_INTERVAL.lock().unwrap();
    *interval = new_change_interval;

    let store = app_handle.store("store.json").expect("store not found");
    store.set("wallpaper_interval", json!(new_change_interval));
    store.save().expect("failed to save store");

    let (lock, cvar) = GLOBAL_CONDVAR.as_ref();
    let mut restart_flag = lock.lock().unwrap();
    *restart_flag = true;
    cvar.notify_one();
}

#[command]
fn get_wallpaper_change_interval() -> u64 {
    let interval = GLOBAL_WALLPAPER_CHANGE_INTERVAL.lock().unwrap();
    *interval
}

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

fn get_images_dir() -> Result<PathBuf, String> {
    let app_dirs = AppDirs::new(Some("wallpaper-changer"), true)
        .ok_or("Failed to determine application directories")?;
    let images_dir = app_dirs.data_dir.join("images");

    if !images_dir.exists() {
        std_fs::create_dir_all(&images_dir)
            .map_err(|e| format!("Failed to create images directory: {}", e))?;
    }

    Ok(images_dir)
}

#[command]
async fn upload_files(files: Vec<FileData>) -> Result<String, String> {
    println!("Received {} files for upload", files.len());

    let images_dir =
        get_images_dir().map_err(|e| format!("Failed to get images directory: {}", e))?;
    println!("Images directory: {:?}", images_dir);

    for file in files {
        println!(
            "Processing file: {} (size: {} bytes)",
            file.name,
            file.data.len()
        );

        let file_path = images_dir.join(&file.name);
        println!("Writing to path: {:?}", file_path);

        match std_fs::File::create(&file_path) {
            Ok(mut file_handle) => {
                file_handle
                    .write_all(&file.data)
                    .map_err(|e| format!("Failed to write file '{}': {}", file.name, e))?;
                println!("Successfully wrote file: {}", file.name);
            }
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
async fn open_images_directory() -> Result<String, String> {
    let images_dir = get_images_dir()?;
    open::that(&images_dir).map_err(|e| format!("Failed to open images directory: {}", e))?;
    Ok(format!("Opened images directory: {:?}", images_dir))
}

#[command]
async fn get_files() -> Result<Vec<FileInfo>, String> {
    println!("Fetching list of images");

    let images_dir =
        get_images_dir().map_err(|e| format!("Failed to get images directory: {}", e))?;
    println!("Reading directory: {:?}", images_dir);

    let mut files = Vec::new();

    let mut dir_entries = tokio_fs::read_dir(&images_dir)
        .await
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    while let Some(entry) = dir_entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Build the URL from the local image server.
                let file_url = format!("http://127.0.0.1:8080/{}", name);
                files.push(FileInfo {
                    name: name.to_string(),
                    data: file_url,
                });
                println!("Found file: {}", name);
            }
        }
    }

    println!("Found {} files", files.len());
    Ok(files)
}

#[command]
async fn delete_image(file_name: String) -> Result<String, String> {
    println!("Attempting to delete image: {}", file_name);
    let images_dir = get_images_dir()?;
    println!("Images directory: {:?}", images_dir);
    let full_path = images_dir.join(&file_name);
    println!("Full file path to delete: {:?}", full_path);

    if full_path.is_file() {
        std_fs::remove_file(&full_path)
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
async fn delete_all_images() -> Result<String, String> {
    println!("Attempting to clear images directory");
    let images_dir = get_images_dir()?;
    println!("Clearing directory: {:?}", images_dir);

    let dir_entries =
        std_fs::read_dir(&images_dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in dir_entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    match std_fs::remove_file(&path) {
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
    let entries = std_fs::read_dir(&images_dir)
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
    let (lock, cvar) = GLOBAL_CONDVAR.as_ref();
    let mut restart_flag = lock.lock().unwrap();

    loop {
        while !*restart_flag {
            match set_random_wallpaper() {
                Ok(msg) => println!("{}", msg),
                Err(e) => eprintln!("Error setting wallpaper: {}", e),
            }

            let interval = *GLOBAL_WALLPAPER_CHANGE_INTERVAL.lock().unwrap();
            let result = cvar
                .wait_timeout(restart_flag, Duration::from_secs(interval))
                .unwrap();

            // If timeout occurs, continue to update wallpaper
            if result.1.timed_out() {
                restart_flag = result.0;
                break;
            }

            restart_flag = result.0;
        }

        *restart_flag = false;
    }
}

pub fn run() {
    println!("Starting Tauri application...");

    thread::spawn(|| {
        let images_dir = get_images_dir().expect("Could not determine images directory");
        println!("Starting image server serving directory: {:?}", images_dir);

        let server = HttpServer::new(move || {
            App::new().service(
                fs::Files::new("/", images_dir.clone())
                    // Optionally, remove file listings for production use:
                    .show_files_listing(),
            )
        })
            .bind("127.0.0.1:8080")
            .expect("Failed to bind to image server port")
            .run();

        println!("Started image server at 127.0.0.1:8080");

        actix_web::rt::System::new()
            .block_on(server)
            .expect("TODO: panic message");
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let store = app.store("store.json")?;
            let stored_interval = store
                .get("wallpaper_interval")
                .and_then(|v| v.as_u64())
                .unwrap_or(300);
            {
                let mut interval = GLOBAL_WALLPAPER_CHANGE_INTERVAL.lock().unwrap();
                *interval = stored_interval;
            }
            println!("Loaded wallpaper interval: {}", stored_interval);

            thread::spawn(|| {
                start_wallpaper_update();
            });

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let change_i = MenuItem::with_id(app, "change wallpaper", "Change Wallpaper", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_i, &change_i, &quit_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    "change wallpaper" => {
                        println!("change wallpaper menu item was clicked");
                        let _ = set_random_wallpaper();
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
                println!("Hiding window and minimizing to tray");
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            upload_files,
            delete_image,
            delete_all_images,
            open_images_directory,
            get_files,
            set_random_wallpaper,
            modify_wallpaper_change_interval,
            get_wallpaper_change_interval
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}