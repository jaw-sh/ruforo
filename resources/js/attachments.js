import { blake3 } from 'hash-wasm';

(async function () {
    console.log(await blake3('foo'));
})();

document.addEventListener("DOMContentLoaded", function() {
    document.querySelector('.attachment-upload').addEventListener('click', async function uploadFile(event) {
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
