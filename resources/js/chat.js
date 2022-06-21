document.addEventListener("DOMContentLoaded", function () {
    let ws = null;
    let room = null;
    let messageHoverEl = null;
    let userHover = null;
    let scrollEl = document.getElementById('chat-scroller');
    let lastScrollPos = 0;

    function messageAddEventListeners(element) {
        if (Object.keys(element.dataset).indexOf('author') > -1) {
            element.addEventListener('mouseenter', messageMouseEnter);
            element.addEventListener('mouseleave', messageMouseLeave);
        }

        let authorEl = element.querySelector('.author');
        if (authorEl !== null) {
            authorEl.addEventListener('click', usernameClick);
        }

        Array.from(element.querySelectorAll('.username')).forEach(function (usernameEl) {
            usernameEl.addEventListener('click', usernameClick);
            usernameEl.addEventListener('mouseenter', usernameEnter);
            usernameEl.addEventListener('mouseleave', usernameLeave);
        });
    }

    function messageDelete(message) {
        let el = document.getElementById(`chat-message-${message}`);
        let prev = el.previousElementSibling;
        let next = el.nextElementSibling;

        if (prev !== null && next !== null && prev.dataset.author == prev.dataset.author) {
            // Allow to break into new groups if too much time has passed.
            let timeLast = parseInt(prev.dataset.received, 10);
            let timeNext = parseInt(next.dataset.received, 10);
            if (timeNext - timeLast < 30000) {
                next.classList.add("chat-message--hasParent");
            }
        }

        el.remove();
        lastScrollPos = 0;
    }

    function messageMouseEnter(event) {
        var author = parseInt(this.dataset.author, 10);

        // Are we already hovering over something?
        if (messageHoverEl !== null) {
            // Is it the same message?
            if (this == messageHoverEl) {
                // We don't need to do anything.
                return true;
            }

            // Is it by the same author?
            if (author === parseInt(messageHoverEl.dataset.author, 10)) {
                // Great, we don't need to do anything.
                //messageHoverEl = $msg;
                //chat.$msgs.children().removeClass(chat.classes.highlightHover);
                //$msg.addClass(chat.classes.highlightHover);
                return true;
            }
        }

        messageHoverEl = this;

        Array.from(document.querySelectorAll('.chat-message--highlightAuthor')).forEach(function (el) {
            el.classList.remove('chat-message--highlightAuthor');
        });

        Array.from(document.querySelectorAll(`.chat-message[data-author='${author}']`)).forEach(function (el) {
            el.classList.add('chat-message--highlightAuthor');
        });
    }

    function messageMouseLeave(event) {
        // We only need to do anything if we're hovering over this message.
        // If we moved between messages, this work is already done.
        if (messageHoverEl !== null && messageHoverEl == this) {
            // We are off of any message, so remove the hovering classes.
            messageHoverEl = null;
            Array.from(document.querySelectorAll('.chat-message--highlightAuthor')).forEach(function (el) {
                el.classList.remove('chat-message--highlightAuthor');
            });
        }
    }

    function messagePush(message, author) {
        if (typeof message === 'string') {
            message = { message: message };
        }

        let messagesEl = document.getElementById('chat-messages');
        let template = document.getElementById('tmp-chat-message').content.cloneNode(true);
        let timeNow = new Date();

        template.querySelector('.message').innerHTML = message.message;
        template.children[0].dataset.received = timeNow.getTime();

        if (author) {
            template.children[0].id = `chat-message-${message.message_id}`;
            template.children[0].dataset.author = author.id;

            // Ignored poster?
            if (APP.user.ignored_users.includes(author.id)) {
                template.children[0].classList.add("chat-message--isIgnored");
            }

            // Group consequtive messages by the same author.
            let lastChild = messagesEl.lastElementChild;
            if (lastChild !== null && lastChild.dataset.author == author.id) {
                // Allow to break into new groups if too much time has passed.
                let timeLast = new Date(parseInt(lastChild.dataset.received, 10));
                if (timeNow.getTime() - timeLast.getTime() < 30000) {
                    template.children[0].classList.add("chat-message--hasParent");
                }
            }

            // Add meta details
            let authorEl = template.querySelector('.author');
            authorEl.innerHTML = author.username;
            authorEl.dataset.id = author.id;

            Array.from(template.querySelectorAll('.timestamp')).forEach(function (el) {
                let time = new Date(message.message_date * 1000);
                let hours = time.getHours();
                let minutes = String(time.getMinutes()).padStart(2, '0');

                el.setAttribute('datetime', message.message_date);

                if (el.classList.contains('relative')) {
                    let dayThen = new Date(message.message_date * 1000).setHours(0, 0, 0, 0);
                    let dayNow = new Date().setHours(0, 0, 0, 0);

                    // Same day, only show clock
                    if (dayThen == dayNow) {
                        el.innerHTML = time.toLocaleTimeString();
                    }
                    // Different days, show date too.
                    else {
                        el.innerHTML = time.toLocaleDateString() + " " + time.toLocaleTimeString()
                    }
                }
                else {
                    el.innerHTML = (hours % 12) + ":" + minutes + " " + (hours >= 12 ? "PM" : "AM");
                }
            });

            // Add left-content details
            if (author.avatar_url.length > 0) {
                template.querySelector('.avatar').setAttribute('src', author.avatar_url);
            }
            else {
                template.querySelector('.avatar').remove();
            }

            // Add right-content details
            template.querySelector('.report').setAttribute('href', `/chat/messages/${message.message_id}/report`);
        }
        else {
            template.children[0].classList.add("chat-message--systemMsg");
            template.querySelector('.meta').remove();
            template.querySelector('.left-content').remove();
            //template.querySelector('.right-content').remove();
        }

        // TODO: FIND SOMETHING BETTER FOR THIS
        // Force set URLs to target new tab.
        Array.from(template.querySelectorAll('.tagUrl')).forEach(function (el) {
            el.target = "_blank";
        });

        // Check tagging.
        if (message.message.includes(`@${APP.user.username}`)) {
            template.children[0].classList.add("chat-message--highlightYou");
        }

        let el = messagesEl.appendChild(template.children[0]);
        messageAddEventListeners(el);

        // Prune oldest messages.
        while (messagesEl.children.length > 200) {
            messagesEl.children[0].remove();
            lastScrollPos = 0;
        }

        messagesEl.children[0].classList.remove("chat-message--hasParent");

        // Scroll down.
        scrollToNew();

        return el;
    }

    function messageSend(message) {
        ws.send(message);
    }

    function messagesReceive(data) {
        // Try to parse JSON data.
        try {
            json = JSON.parse(data);

            if (json.hasOwnProperty('messages')) {
                json.messages.forEach(message => messagePush(message, message.author));
            }

            if (json.hasOwnProperty('delete')) {
                json.delete.forEach(message => messageDelete(message));
            }
        }
        // Not valid JSON, default
        catch (error) {
            messagePush({ message: data }, null);
        }
    }

    function messagesDelete() {
        let messagesEl = document.getElementById('chat-messages');
        while (messagesEl.firstChild) {
            messagesEl.removeChild(messagesEl.firstChild);
        }
    }

    function roomJoin(id) {
        if (Number.isInteger(id) && id > 0) {
            scrollEl.classList.remove('ScrollAnchored');
            scrollEl.classList.add('ScrollAnchorConsume');
            messagesDelete();
            messageSend(`/join ${id}`);
            //document.getElementById("chat-input").focus({ preventScroll: true });
            return true;
        }

        console.log(`Attempted to join a room with an ID of ${room_id}`);
        return false;
    }

    function roomJoinByHash() {
        let room_id = parseInt(window.location.hash.substring(1), 10);

        if (room_id > 0) {
            return roomJoin(room_id);
        }

        return false;
    }

    function scrollerScroll(event) {
        const clampHeight = 64; // margin of error

        // if last scrollTop is lower (greater) than current scroll top,
        // we have scrolled down.
        if (lastScrollPos > this.scrollTop) {
            if (!this.classList.contains("ScrollAnchorConsume")) {
                this.classList.add('ScrollAnchored');
            }
            else {
                this.classList.remove('ScrollAnchorConsume');
            }
        }
        // if we've scrolled down and we are very close to the bottom
        // based on the height of the viewport, lock it in
        else if (this.offsetHeight + this.scrollTop >= this.scrollHeight - clampHeight) {
            this.classList.remove('ScrollAnchored');
        }

        lastScrollPos = this.scrollTop;
    }

    function scrollToNew() {
        if (!scrollEl.classList.contains('ScrollAnchored')) {
            scrollEl.scrollTo(0, scrollEl.scrollHeight);
        }
    }

    function usernameClick(event) {
        // TODO: Replace with Dialog like Discord?
        let input = document.getElementById('chat-input')
        input.value += `@${this.textContent}, `;
        input.setSelectionRange(input.value.length, input.value.length);
        input.focus();

        event.preventDefault();
        return false;
    }

    function usernameEnter(event) {
        var id = parseInt(this.dataset.id, 10);

        if (userHover === id) {
            return true;
        }

        userHover = id;

        Array.from(document.querySelectorAll('.chat-message--highlightUser')).forEach(function (el) {
            el.classList.remove('chat-message--highlightUser');
        });
        Array.from(document.querySelectorAll(`[data-author='${id}']`)).forEach(function (el) {
            el.classList.add('chat-message--highlightUser');
        });
    }

    function usernameLeave(event) {
        var id = parseInt(this.dataset.id, 10);

        // Are we hovering over the same message still?
        // This stops unhovering when moving between hover targets.
        if (userHover === id) {
            userHover = null;
            Array.from(document.querySelectorAll('.chat-message--highlightUser')).forEach(function (el) {
                el.classList.remove('chat-message--highlightUser');
            });
        }
    }

    function websocketConnect() {
        // TODO: Make this something practical.
        // fixes cross-domain issues that the forum currently enjoy
        // transform "wss://mysite.us/rust-chat" to "wss://mysite.eu/rust-chat" when on mysite.eu, for instance.
        let sneed = new URL(APP.chat_ws_url);
        sneed.hostname = window.location.hostname;
        sneed.protocol = window.location.protocol == "http:" ? "ws:" : "wss:";

        ws = new WebSocket(sneed.href);
        messagePush("Connecting to SneedChat...");

        ws.addEventListener('close', function (event) {
            messagePush("Connection lost. Please wait - attempting reestablish");
            setTimeout(websocketConnect, 3000);
        });

        ws.addEventListener('error', function (event) {
            console.log(event);
        });

        ws.addEventListener('message', function (event) {
            messagesReceive(event.data);
        });

        ws.addEventListener('open', function (event) {
            if (room === null) {
                if (!roomJoinByHash()) {
                    messagePush("Connected! You may now join a room.");
                }
                else {
                    messagePush("Connected!");
                }
            }
            else {
                messagePush(`Connected to <em>${room.title}</em>!`);
            }
        });
    }


    // Room buttons
    //document.getElementById('chat-rooms').addEventListener('click', function (event) {
    //    let target = event.target;
    //    if (target.classList.contains('chat-room')) {
    //        let room_id = parseInt(target.dataset.id, 10);
    //
    //        if (!isNaN(room_id) && room_id > 0) {
    //            messageSend(`/join ${room_id}`);
    //        }
    //        else {
    //            console.log(`Attempted to join a room with an ID of ${room_id}`);
    //        }
    //    }
    //});

    // Scroll window
    scrollEl.addEventListener('scroll', scrollerScroll);
    //scrollEl.classList.add('ScrollLocked');
    setInterval(scrollToNew, 32);

    // Form
    document.getElementById('chat-input').addEventListener('keydown', function (event) {
        if (event.key === "Enter" || event.keyCode == 13) {
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

    document.getElementById('chat-submit').addEventListener('click', function (event) {
        event.preventDefault();
        let input = document.getElementById('chat-input');

        messageSend(input.value);

        input.value = "";
        input.focus({ preventScroll: true });
        return false;
    });


    window.addEventListener('hashchange', roomJoinByHash, false);
    window.addEventListener('resize', function (event) {
        if (!scrollEl.classList.contains("ScrollAnchor")) {
            scrollEl.classList.add("ScrollAnchorConsume");
        }
        scrollToNew();
    });


    websocketConnect();
});
