const { invoke } = window.__TAURI__.core;

const maxFilesToRender = 100;

async function loadImages()
{
    try
    {
        const gallery = document.getElementById('image-gallery');
        const fileCount = await invoke('count_files');

        gallery.innerHTML = '';

        if (fileCount > maxFilesToRender)
        {
            const imageDirButton = document.createElement('button');
            imageDirButton.textContent = 'Open Image Directory';
            imageDirButton.addEventListener('click', () => { invoke('open_images_directory') })

            gallery.innerHTML = '';
            gallery.appendChild(imageDirButton);

            const buttonWrapper = document.querySelector('.button-wrapper');
            buttonWrapper.style.marginBottom = '1rem';

            alert(`You have ${fileCount} files a maximum of ${maxFilesToRender} files are supported for rendering. If you would like to view the source file directory click the above. The desktop wallpaper switching  functionily will still function no matter the number of files present in the files directory.`)

            return;
        }

        gallery.innerHTML = 'Loading images...';
        const files = await invoke('get_files');
        gallery.innerHTML = '';

        files.forEach(file =>
        {
            const imageContainer = document.createElement('div');
            imageContainer.className = 'image-container';

            const img = document.createElement('img');
            img.src = `data:image/png;base64,${file.thumbnail}`;
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

async function uploadFiles(files)
{
    const CHUNK_SIZE = 8;
    const totalFiles = files.length;

    let processed = 0;

    while (processed < totalFiles)
    {
        const chunk = Array.from(files).slice(processed, processed + CHUNK_SIZE);
        const fileArray = await Promise.all(chunk.map(async file =>
        {
            const fileBuffer = await file.arrayBuffer();
            return {
                name: file.name.replaceAll(" ", "_"),
                data: Array.from(new Uint8Array(fileBuffer)),
            };
        }));

        try
        {
            await invoke("upload_files", { files: fileArray });
            processed += chunk.length;
        } catch (error)
        {
            alert(`Error uploading files: ${error}`);
            break;
        }
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