import { blake3 } from 'hash-wasm';

document.addEventListener("DOMContentLoaded", function() {
    document.querySelector('.attachment-input').addEventListener('change', async function (event) {
        let file = event.target.files[0];
        if (file) {
            let reader = new FileReader();

            reader.onload = async function (readerEvent) {
                let hash = await blake3(new Uint8Array(readerEvent.target.result));
            }

            reader.onerror = function (readerEvent) {
                console.log("error reading file");
            }

            reader.readAsArrayBuffer(file);
        }
    });

    document.querySelector('.attachment-upload').addEventListener('click', async function (event) {
        event.preventDefault();

        let formData = new FormData();
        formData.append("file", document.querySelector('.attachment-input').files[0]);

        let response = await fetch('/fs/upload-file', {
            method: "POST",
            body: formData
        });
        return false; // prevent default
    });
});
