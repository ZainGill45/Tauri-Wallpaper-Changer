const { invoke } = window.__TAURI__.core;

document.addEventListener("DOMContentLoaded", () =>
{
    const dropzone = document.getElementById('dropzone');
    const fileInput = document.getElementById('file-input');
    const deleteAllImages = document.getElementById('delete-all-images-button');
    const resetWallpaperCounter = document.getElementById('reset-wallpaper-counter-button');

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
    });
    fileInput.addEventListener('change', async (event) =>
    {
        event.preventDefault();

        const files = fileInput.files;

        if (files.length === 0)
        {
            alert("No files selected.");
            return;
        }

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
    });

    deleteAllImages.addEventListener('click', async () =>
    {
        await invoke('delete_all_images');

        imageElements = document.querySelectorAll(".image-container");

        for (let i = 0; i < imageElements.length; i++)
            imageElements[i].remove();
    });
    resetWallpaperCounter.addEventListener('click', () =>
    {
        invoke('set_random_wallpaper');
    });

    loadImages();
});

async function loadImages()
{
    try
    {
        const gallery = document.getElementById('image-gallery');
        gallery.innerHTML = 'Loading images...';

        const files = await invoke('get_files');

        gallery.innerHTML = '';

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

    } catch (error)
    {
        console.error('Error loading images:', error);
        document.getElementById('image-gallery').innerHTML =
            `Error loading images: ${error}`;
    }
}

async function deleteImage(fileName, element)
{
    try
    {
        const response = await invoke('delete_image', { fileName: fileName });

        if (response && response.includes('Successfully deleted'))
        {
            alert('Image has been deleted successfully.');
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

