document.addEventListener("DOMContentLoaded", function () {
    // WebSocket
    let ws = new WebSocket("ws://xf.localhost/rust-chat");
    pushMessage("Connecting...");

    ws.addEventListener('close', function (event) {
        console.log(event);
        pushMessage("Connection closed.");
    });

    ws.addEventListener('error', function (event) {
        console.log(event);
    });

    ws.addEventListener('message', function (event) {
        console.log(event);
        let author = null;
        let message = null;

        // Try to parse JSON data.
        try {
            let json = JSON.parse(event.data);
            console.log(json);
            author = json.author;
            message = json.message;
        }
        // Not valid JSON, default
        catch (error) {
            message = event.data;
        }
        // Push whatever we got to chat.
        finally {
            pushMessage(message, author);
        }
    });

    ws.addEventListener('open', function (event) {
        console.log(event);
        pushMessage("Connected!");
    });

    function pushMessage(message, author) {
        let messages = document.getElementById('messages');
        let template = document.getElementById('tmp-chat-message').content.cloneNode(true);

        template.querySelector('.message').innerHTML = message;

        if (typeof author === 'object' && author !== null) {
            template.children[0].dataset.author = author.id;
            template.children[0].dataset.received = new Date().getTime();

            if (messages.lastElementChild !== null && messages.lastElementChild.dataset.author == author.id) {
                template.children[0].classList.add("chat-message--hasParent");
            }

            template.querySelector('.author').innerHTML = author.username;
            template.querySelector('.avatar').setAttribute('src', `/data/avatars/m/${Math.floor(author.id / 1000)}/${author.id}.jpg?${author.avatar_date}`);
        }
        else {
            template.querySelector('.avatar').remove();
        }

        messages.appendChild(template);
    }

    // Form
    document.getElementById('message-input').addEventListener('keydown', function (event) {
        if (event.which === 13) {
            event.preventDefault();
            ws.send(this.value);
            this.value = "";
            return false;
        }
    });
});
