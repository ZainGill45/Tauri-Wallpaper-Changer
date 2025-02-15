const { invoke } = window.__TAURI__.core;

document.addEventListener("DOMContentLoaded", () =>
{
    const dropzone = document.getElementById('dropzone');
    const fileInput = document.getElementById('fileInput');
    const gallery = document.getElementById('image-gallery');

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
            const response = await invoke("upload_files", { files: fileArray });
        } catch (error)
        {
            alert("Error uploading files: " + error);
        }
    });

    const resizeObserver = new ResizeObserver((entries) =>
    {
        for (let entry of entries)
        {
            const galleryWidth = entry.contentRect.width;
            dropzone.style.maxWidth = `${galleryWidth}px`;
            console.log("running")
        }
    });

    resizeObserver.observe(gallery);

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
            img.src = './assets/images/' + file.name;
            img.alt = file.name;

            const imgOverlay = document.createElement('div');
            imgOverlay.classList.add('overlay');
            const imgSpan = document.createElement('span');
            imgSpan.classList.add('icon');
            const spanImg = document.createElement('img');
            spanImg.src = '/assets/trash.png';

            imgSpan.appendChild(spanImg)

            imgOverlay.appendChild(imgSpan);

            imageContainer.appendChild(img);
            imageContainer.appendChild(imgOverlay);

            gallery.appendChild(imageContainer);
        });

    } catch (error)
    {
        console.error('Error loading images:', error);
        document.getElementById('image-gallery').innerHTML =
            `Error loading images: ${error}`;
    }
}

function clearImages()
{
    invoke('clear_images')
        .then(response => alert(response))
        .catch(error => alert('Failed to clear images:', error));
}