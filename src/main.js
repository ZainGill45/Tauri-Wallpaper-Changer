const { invoke } = window.__TAURI__.core;

document.addEventListener("DOMContentLoaded", () =>
{
    const imageUploadElement = document.querySelector("#image-upload");

    document.getElementById('refresh-button')?.addEventListener('click', loadImages);

    loadImages();

    imageUploadElement.addEventListener("change", async (event) =>
    {
        event.preventDefault();

        const files = imageUploadElement.files;

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
});

async function loadImages()
{
    try
    {
        // Show loading state
        const gallery = document.getElementById('image-gallery');
        gallery.innerHTML = 'Loading images...';

        // Get images from Rust backend
        const files = await invoke('get_files');

        // Clear loading message
        gallery.innerHTML = '';

        // Create image elements for each file
        files.forEach(file =>
        {
            console.log(file);

            const imageContainer = document.createElement('div');
            imageContainer.className = 'image-container';

            const img = document.createElement('img');
            img.src = './assets/images/' + file.name;
            img.alt = file.name;

            const formattedString = file.name.replaceAll("_", " ");
            const dotIndex = file.name.indexOf('.');

            const name = document.createElement('p');
            name.textContent = dotIndex != -1 ? formattedString.slice(0, dotIndex) : formattedString;

            imageContainer.appendChild(img);
            imageContainer.appendChild(name);
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