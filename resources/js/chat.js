document.addEventListener("DOMContentLoaded", function () {
    // WebSocket
    const CHAT_URL = "ws://xf.localhost/rust-chat";
    let ws = new WebSocket(CHAT_URL);
    pushMessage("Connecting...");

    ws.addEventListener('close', function (event) {
        pushMessage("Connection closed by remote server.");
    });

    ws.addEventListener('error', function (event) {
        console.log(event);
    });

    ws.addEventListener('message', function (event) {
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
        let timeNow = new Date();

        template.querySelector('.message').innerHTML = message;
        template.children[0].dataset.received = timeNow.getTime();

        // Set the relative timestamp
        let timestamp = template.querySelector('time');
        timestamp.setAttribute('datetime', timeNow.toISOString());
        timestamp.innerHTML = "Just now";

        if (typeof author === 'object' && author !== null) {
            template.children[0].dataset.author = author.id;

            // Group consequtive messages by the same author.
            let lastChild = messages.lastElementChild;
            if (lastChild !== null && lastChild.dataset.author == author.id) {
                // Allow to break into new groups if too much time has passed.
                let timeLast = new Date(parseInt(lastChild.dataset.received, 10));
                if (timeNow.getTime() - timeLast.getTime() < 30000) {
                    template.children[0].classList.add("chat-message--hasParent");
                }
            }

            template.querySelector('.author').innerHTML = author.username;
            template.querySelector('.avatar').setAttribute('src', `/data/avatars/m/${Math.floor(author.id / 1000)}/${author.id}.jpg?${author.avatar_date}`);
        }
        else {
            template.querySelector('.meta').remove();
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
