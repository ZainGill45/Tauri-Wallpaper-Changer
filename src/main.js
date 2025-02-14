const { invoke } = window.__TAURI__.core;

const uploadForm = document.getElementById("uploadForm");
const resultDiv = document.getElementById("result");

uploadForm.addEventListener("submit", async (event) =>
{
    event.preventDefault();

    const fileInput = document.getElementById("fileInput");
    const files = fileInput.files;

    if (files.length === 0)
    {
        resultDiv.textContent = "No files selected.";
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
        resultDiv.textContent = response;
    } catch (error)
    {
        console.error("Error uploading files:", error);
        resultDiv.textContent = "Failed to upload files.";
    }
});