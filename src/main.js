class LoadingOverlay
{
    constructor()
    {
        this.overlay = document.querySelector('.loading-overlay');
        this.isActive = false;

        this.overlay.addEventListener('click', (e) =>
        {
            e.stopPropagation();
        });

        document.addEventListener('keydown', (e) =>
        {
            if (this.isActive)
            {
                e.preventDefault();
            }
        });
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

const { invoke } = window.__TAURI__.core;
const loader = new LoadingOverlay();

async function loadImages()
{
    loader.show();

    try
    {
        const gallery = document.querySelector('.image-gallery');
        const files = await invoke('get_files');

        files.forEach(file =>
        {
            const imageContainer = document.createElement('div');
            imageContainer.className = 'image-container';

            const img = document.createElement('img');
            img.src = `data:image/png;base64,${file.data}`;
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

            const appendedElement = gallery.appendChild(imageContainer);

            appendedElement.addEventListener("click", () =>
            {
                deleteImage(appendedElement.querySelector('img').alt, appendedElement);
            });
        });

        loader.hide();
    } catch (error)
    {
        console.error('Error loading images:', error);

        loader.hide();
    }
}

async function uploadFiles(files)
{
    if (files.length === 0)
    {
        alert("No files selected.");
        return;
    }

    loader.show();

    const fileArray = [];
    for (const file of files)
    {
        const fileBuffer = await file.arrayBuffer();

        fileArray.push({
            name: file.name.replaceAll(" ", "_"),
            data: Array.from(new Uint8Array(fileBuffer)),
        });
    }

    try
    {
        await invoke("upload_files", { files: fileArray });
    } catch (error)
    {
        alert("Error uploading files: " + error);
    }

    loadImages();
}

async function deleteImage(fileName, element)
{
    try
    {
        const response = await invoke('delete_image', { fileName: fileName });

        if (response && response.includes('Successfully deleted'))
        {
            element.remove();
        } else
        {
            alert('Image not found.');
        }
    } catch (error)
    {
        alert('Error deleting image: ' + error);
    }
}

document.addEventListener("DOMContentLoaded", () =>
{
    const dropzone = document.querySelector('#dropzone');
    const fileInput = document.querySelector('#file-input');
    const deleteAllImages = document.querySelector('#delete-all-images-button');
    const resetWallpaperCounter = document.querySelector('#reset-wallpaper-counter-button');
    const openImageDirectory = document.querySelector('#open-image-directory-button');

    dropzone.addEventListener('click', () => fileInput.click());
    dropzone.addEventListener('dragover', (event) =>
    {
        event.preventDefault();
        dropzone.style.backgroundColor = '#f3f4f6';
    });
    dropzone.addEventListener('dragleave', () =>
    {
        dropzone.style.backgroundColor = '';
    });
    dropzone.addEventListener("drop", async (event) =>
    {
        event.preventDefault();

        const files = event.dataTransfer.files;

        if (files.length === 0)
        {
            alert("No files selected.");
            return;
        }

        await uploadFiles(files);
    });
    fileInput.addEventListener('change', async (event) =>
    {
        event.preventDefault();
        if (fileInput.files.length === 0)
        {
            alert("No files selected.");
            return;
        }
        await uploadFiles(fileInput.files);
    });

    deleteAllImages.addEventListener('click', async () =>
    {
        await invoke('delete_all_images');

        let imageElements = document.querySelectorAll(".image-container");

        for (let i = 0; i < imageElements.length; i++)
            imageElements[i].remove();
    });
    resetWallpaperCounter.addEventListener('click', () =>
    {
        invoke('set_random_wallpaper');
    });
    openImageDirectory.addEventListener('click', () =>
    {
        invoke('open_images_directory');
    });
});

