document.addEventListener("DOMContentLoaded", function () {
    // WebSocket
    const CHAT_URL = "ws://xf.localhost/rust-chat";

    let ws = new WebSocket(CHAT_URL);
    let room = null;

    messagePush("Connecting to SneedChat...");

    ws.addEventListener('close', function (event) {
        messagePush("Connection closed by remote server.");
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
            author = json.author;
            message = json.message;
        }
        // Not valid JSON, default
        catch (error) {
            message = event.data;
        }
        // Push whatever we got to chat.
        finally {
            messagePush(message, author);
        }
    });

    ws.addEventListener('open', function (event) {
        if (room === null) {
            messagePush("Connected! You may now join a room.");
        }
        else {
            messagePush(`Connected to <em>${room.title}</em>!`);
        }
    });

    function messagePush(message, author) {
        let messages = document.getElementById('chat-messages');
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
            template.querySelector('.left-content').remove();
            template.querySelector('.right-content').remove();
        }

        messages.appendChild(template);
        scrollToNew();
    }

    function messageSend(message) {
        ws.send(message);
    }

    function scrollToNew() {
        let scroller = document.getElementById('chat-scroller');
        scroller.scrollTo(0, scroller.scrollHeight);
    }

    // Room buttons
    document.getElementById('chat-rooms').addEventListener('click', function (event) {
        let target = event.target;
        if (target.classList.contains('chat-room')) {
            let room_id = parseInt(target.dataset.id, 10);

            if (!isNaN(room_id) && room_id > 0) {
                messageSend(`/join ${room_id}`);
            }
            else {
                console.log(`Attempted to join a room with an ID of ${room_id}`);
            }
        }
    });

    // Form
    document.getElementById('chat-input').addEventListener('keydown', function (event) {
        if (event.key === "Enter") {
            event.preventDefault();

            //let formData = new FormData(this.parentElement);
            //let formProps = Object.fromEntries(formData);
            //
            //messageSend(JSON.stringify(formProps));
            messageSend(this.value);

            this.value = "";
            return false;
        }
    });
});
