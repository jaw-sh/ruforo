import { blake3 } from 'hash-wasm';

document.addEventListener("DOMContentLoaded", function () {
    function attachmentEventListeners(element) {
        let inputEl = document.querySelector('.attachment-input');
        if (inputEl !== null) {
            inputEl.addEventListener('change', async function (event) {
                let file = event.target.files[0];
                if (file) {
                    let reader = new FileReader();

                    reader.onload = async function (readerEvent) {
                        let hash = await blake3(new Uint8Array(readerEvent.target.result));
                        let formData = new FormData();
                        formData.append("hash", hash);

                        let response = await fetch('/fs/check-file', {
                            method: "POST",
                            headers: {
                                'Content-Type': 'application/json'
                            },
                            body: JSON.stringify({
                                hash: hash,
                            }),
                        });

                        console.log(response);
                    }

                    reader.onerror = function (readerEvent) {
                        console.log("error reading file");
                    }

                    reader.readAsArrayBuffer(file);
                }
            });
        }

        let uploadEl = document.querySelector('.attachment-upload');
        if (uploadEl !== null) {
            uploadEl.addEventListener('click', async function (event) {
                event.preventDefault();

                let formData = new FormData();
                formData.append("file", document.querySelector('.attachment-input').files[0]);

                let response = await fetch('/fs/upload-file', {
                    method: "POST",
                    body: formData
                });
                return false; // prevent default
            });
        }
    }

    attachmentEventListeners();
});
