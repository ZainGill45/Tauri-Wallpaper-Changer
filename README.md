# Wallpaper Changer App

## Overview

The **Wallpaper Changer App** is designed to randomly set the same wallpaper across multiple monitors at regular intervals. It was created to address the lack of a simple way to achieve this functionality on Windows systems. The app was built using **Tauri** (for creating a lightweight desktop application) and **Rust** (for backend operations), offering an opportunity to explore and learn both technologies.

## Purpose

I created this app because I couldn't find a solution to randomly change the wallpaper across all my monitors using the same wallpaper at regular intervals. Despite trying several methods and spending multiple days looking for a solution, I couldn’t find one. So, I built this app. It allows me to automatically rotate wallpapers on my desktop every minute, keeping the experience fresh with minimal effort.

The app also supports managing wallpapers: uploading, deleting, and viewing images.

Note that this project was created in one day, so performance is a big issue, especially with base64 encoding for a large number of images. If you plan to use a large image library, it’s recommended to manually copy and paste the images into the `AppData/wallpaper-changer/images` directory. It is also recommended to only have maximum of 100 images that total 256mb for optimal performance.

## Features

- **Random Wallpaper Rotation:** Sets a random wallpaper from the images folder every minute.
- **Upload Wallpapers:** Allows users to upload images to the app for use as wallpapers.
- **Delete Wallpapers:** Users can delete individual images or clear all images from the app's image directory.
- **Cross-Monitor Support:** Works on multiple monitors by setting the same wallpaper on all connected displays.

## Technologies Used

- **Tauri:** Lightweight framework for building desktop applications with web technologies.
- **Rust:** The backend is written in Rust, a systems programming language known for its speed and safety.
- **Windows API:** Used to interact with the system's wallpaper settings.
- **Base64 Encoding:** Used to upload images as base64 encoded strings for portability.

## Installation

### Use the Installer in the Releases

1. Run the installer, which will set everything up, using `AppData\Local\wallpaper-changer` as the base directory where all the images are stored.

### Build from Source

1. Install Cargo, Rust, and Tauri dependencies.
2. Run `cargo tauri build`.
3. The output binaries will be located in `src-tauri/target/release/bundle`.

### Prerequisites

- [Rust](https://www.rust-lang.org/learn/get-started) installed on your machine.
- [Node.js](https://nodejs.org/) (for Tauri setup).
