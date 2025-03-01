function waitForTauri()
{
    return new Promise((resolve) =>
    {
        const interval = setInterval(() =>
        {
            if (window.__TAURI__)
            {
                clearInterval(interval);
                resolve();
            }
        }, 50);
    });
}

let invoke;

class LoadingOverlay
{
    constructor()
    {
        this.overlay = document.querySelector('.loading-overlay');
        this.isActive = false;
        this.overlay.addEventListener('click', e => e.stopPropagation());
        document.addEventListener('keydown', e =>
        {
            if (this.isActive) e.preventDefault();
        });
    }

    show()
    {
        this.isActive = true;
        document.body.classList.add('no-scroll');
        document.body.classList.add('no-select');
        this.overlay.classList.add('active');
    }

    hide()
    {
        this.isActive = false;
        document.body.classList.remove('no-scroll');
        document.body.classList.remove('no-select');
        this.overlay.classList.remove('active');
    }
}
class FormOverlay
{
    constructor()
    {
        this.overlay = document.querySelector('.timer-overlay')
        this.isActive = false;
    }

    show()
    {
        this.isActive = true;
        document.body.classList.add('no-scroll');
        this.overlay.classList.add('active');
    }

    hide()
    {
        this.isActive = false;
        document.body.classList.remove('no-scroll');
        this.overlay.classList.remove('active');
    }
}
class AlertDialog
{
    constructor()
    {
        this.overlay = document.querySelector('.alert-overlay');
        this.dialog = this.overlay.querySelector('.alert-dialog');
        this.titleElement = this.dialog.querySelector('.alert-title');
        this.messageElement = this.dialog.querySelector('.alert-message');
        this.buttonsContainer = this.dialog.querySelector('.alert-buttons');

        this.overlay.addEventListener('click', e =>
        {
            if (e.target === this.overlay) this.hide();
        });

        document.addEventListener('keydown', e =>
        {
            if (e.key === 'Escape' && this.overlay.classList.contains('active')) this.hide();
            if (e.key === 'Space' && this.overlay.classList.contains('active')) this.hide();
        });
    }

    show(title, message, buttons = [])
    {
        this.titleElement.textContent = title;
        this.messageElement.textContent = message;
        this.buttonsContainer.innerHTML = '';
        buttons.forEach(btn =>
        {
            const button = document.createElement('button');
            button.textContent = btn.text;
            button.className = `alert-button ${btn.primary ? 'alert-button-primary' : 'alert-button-secondary'}`;
            button.onclick = () =>
            {
                if (btn.onClick) btn.onClick();
                this.hide();
            };
            this.buttonsContainer.appendChild(button);
        });
        document.body.classList.add('no-scroll');
        document.body.classList.add('no-select');
        this.overlay.classList.add('active');
    }

    hide()
    {
        document.body.classList.remove('no-scroll');
        document.body.classList.remove('no-select');
        this.overlay.classList.remove('active');
    }
}

const loadingOverlay = new LoadingOverlay();
const formOverlay = new FormOverlay();
const alertDialog = new AlertDialog();

async function loadImages()
{
    const { invoke } = window.__TAURI__.core;

    loadingOverlay.show();

    try
    {
        const gallery = document.querySelector('.image-gallery');
        const files = await invoke('get_files');
        files.forEach(file =>
        {
            const imageContainer = document.createElement('div');
            imageContainer.className = 'image-container';
            const img = document.createElement('img');

            img.src = file.data;
            img.alt = file.name;

            const imgOverlay = document.createElement('div');
            imgOverlay.classList.add('overlay');
            const imgSpan = document.createElement('span');
            imgSpan.classList.add('icon');
            const spanImg = document.createElement('img');
            spanImg.src = '/assets/trash.png';

            imgSpan.appendChild(spanImg);
            imgOverlay.appendChild(imgSpan);
            imageContainer.appendChild(img);
            imageContainer.appendChild(imgOverlay);

            const appended = gallery.appendChild(imageContainer);

            appended.addEventListener("click", () =>
            {
                deleteImage(appended.querySelector('img').alt, appended);
            });
        });

        loadingOverlay.hide();

    } catch (error)
    {
        showAlert('Error Loading Images', error);
        loadingOverlay.hide();
    }
}
async function uploadFiles(files)
{
    const { invoke } = window.__TAURI__.core;

    if (!files.length)
    {
        showAlert('No Files Selected', 'Please select at least one file to upload images.');
        return;
    }
    loadingOverlay.show();
    const fileArray = [];
    for (const file of files)
    {
        const fileBuffer = await file.arrayBuffer();
        fileArray.push({ name: file.name.replaceAll(" ", "_"), data: Array.from(new Uint8Array(fileBuffer)) });
    }
    try
    {
        await invoke("upload_files", { files: fileArray });
    } catch (error)
    {
        showAlert('Error uploading files:', error);
    }
    await loadImages();
}
async function deleteImage(fileName, element)
{
    const { invoke } = window.__TAURI__.core;

    try
    {
        const response = await invoke('delete_image', { fileName });
        if (response && response.includes('Successfully deleted'))
        {
            element.remove();
            showAlert('Image Deleted', `Image ${fileName} was deleted.`);
        } else
        {
            showAlert('Error deleting image');
        }
    } catch (error)
    {
        showAlert('Error deleting image: ' + error);
    }
}

document.addEventListener('DOMContentLoaded', async () =>
{
    if (!window.__TAURI__)
    {
        console.warn("Tauri API not found, waiting...");
        await waitForTauri();
    }

    if (!window.__TAURI__.core)
    {
        console.error("Still did not find Tauri API nothing will work.");
        showAlert('Fetal Error', 'Tauri API not found this should not happen please report this error at: https://github.com/ZainGill45/Tauri-Wallpaper-Changer');
    }

    const { invoke } = window.__TAURI__.core;

    const dropzone = document.getElementById('dropzone');
    const fileInput = document.getElementById('file-input');
    const changeIntervalInput = document.getElementById('wallpaper-change-interval-input');
    const changeIntervalBtn = document.getElementById('change-interval-submit-button');
    const deleteAllBtn = document.getElementById('delete-all-images-button');
    const wallpaperBtn = document.getElementById('set-wallpaper-timer-button');
    const openDirBtn = document.getElementById('open-image-directory-button');
    const closeFrmBtn = document.getElementById('timer-overlay-close-button');

    showAlert('General Information', 'For large file uploads, please open the image directory, upload manually, and then restart the application.');

    closeFrmBtn.addEventListener('click', () => formOverlay.hide());
    dropzone.addEventListener('click', () => fileInput.click());
    dropzone.addEventListener('dragover', e =>
    {
        e.preventDefault();
        dropzone.style.backgroundColor = '#f3f4f6';
    });
    dropzone.addEventListener('dragleave', () => dropzone.style.backgroundColor = '');
    dropzone.addEventListener("drop", async e =>
    {
        e.preventDefault();
        const files = e.dataTransfer.files;
        if (!files.length)
        {
            showAlert('No files found.');
            return;
        }
        await uploadFiles(files);
    });
    fileInput.addEventListener('change', async () =>
    {
        if (!fileInput.files.length)
        {
            alert("No files selected.");
            return;
        }
        await uploadFiles(fileInput.files);
    });
    deleteAllBtn.addEventListener('click', async () =>
    {
        await invoke('delete_all_images');
        document.querySelectorAll(".image-container").forEach(el => el.remove());
    });
    changeIntervalBtn.addEventListener('click', async () =>
    {
        formOverlay.hide();

        if (changeIntervalInput.value === '' || changeIntervalInput.value === undefined)
        {
            showAlert('Invalid Interval Value', 'Please select at least one file to upload images.');
            changeIntervalInput.value = '';
        }

        await invoke('modify_wallpaper_change_interval', { newChangeInterval: Number(changeIntervalInput.value) });

        const response = await invoke('get_wallpaper_change_interval');

        showAlert('Wallpaper Interval Changed', `Desktop wallpaper will now switch every ${response} seconds.`);

        changeIntervalInput.value = '';
    });
    wallpaperBtn.addEventListener('click', () =>
    {
        formOverlay.show();
    });
    openDirBtn.addEventListener('click', () => invoke('open_images_directory'));

    await loadImages();
});

function showAlert(title, message)
{
    alertDialog.show(title, message, [{ text: 'Okay', primary: true }]);
}