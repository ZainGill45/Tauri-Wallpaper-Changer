const { invoke } = window.__TAURI__.core;

document.addEventListener("DOMContentLoaded", () =>
{
    const imageUploadElement = document.querySelector("#image-upload");

    imageUploadElement.addEventListener("change", async (event) =>
    {
        event.preventDefault();

        const files = imageUploadElement.files;

        if (files.length === 0)
        {
            alert("No files selected.");
            return;
        }

        // Read files as an array of objects
        const fileArray = [];
        for (const file of files)
        {
            const fileBuffer = await file.arrayBuffer(); // Read file data
            fileArray.push({
                name: file.name,
                data: Array.from(new Uint8Array(fileBuffer)), // Convert buffer to array for serialization
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

function clearImages()
{
    invoke('clear_images')
        .then(response => alert(response))
        .catch(error => alert('Failed to clear images:', error));
}