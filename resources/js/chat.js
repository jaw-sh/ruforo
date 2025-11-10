import { MicroModal } from 'micromodal';

document.addEventListener("DOMContentLoaded", function () {
    let ws = null;
    let room = null;
    let messageHoverEl = null;
    let userHover = null;
    let scrollEl = document.getElementById('chat-scroller');
    let lastScrollPos = 0;
    let userActivityData = {};
    let scrollAnimationFrame = null;

    // Track event listeners for cleanup
    const eventListenerMap = new WeakMap();

    // Reuse a DOM node to decode HTML entities back into plain text
    const decodeHtmlEntities = (() => {
        const textarea = document.createElement('textarea');
        return (value) => {
            if (typeof value !== 'string') {
                return '';
            }

            textarea.innerHTML = value;
            const decoded = textarea.value;
            textarea.value = '';
            return decoded;
        };
    })();

    function inputAddEventListeners(el) {
        // TODO: Add keyDown event listeners?
        // Right now, the functionality for main input and edit input is totally different.
        el.addEventListener('paste', function (event) {
            var text = event.clipboardData.getData('text/plain');

            const sel = window.getSelection();
            if (!sel.rangeCount) {
                return false;
            }
            sel.deleteFromDocument();

            let range = sel.getRangeAt(0);
            let newNode = document.createTextNode(text);
            range.insertNode(newNode);
            range.setStart(this, range.endOffset);

            event.preventDefault();
            return false;
        });
    }

    function inputFocusEnd(el) {
        setTimeout(function () {
            let range = document.createRange();
            range.setStart(el, el.childElementCount + 1);
            range.setEnd(el, el.childElementCount + 1);
            range.collapse(false);

            let sel = window.getSelection();
            sel.removeAllRanges();
            sel.addRange(range);
            el.focus();
        }, 0);
    }

    function messageAddEventListeners(element) {
        const listeners = [];

        if (Object.keys(element.dataset).indexOf('author') > -1) {
            element.addEventListener('mouseenter', messageMouseEnter);
            element.addEventListener('mouseleave', messageMouseLeave);
            listeners.push(
                { target: element, type: 'mouseenter', handler: messageMouseEnter },
                { target: element, type: 'mouseleave', handler: messageMouseLeave }
            );
        }

        let authorEl = element.querySelector('.author');
        if (authorEl !== null) {
            authorEl.addEventListener('click', usernameClick);
            listeners.push({ target: authorEl, type: 'click', handler: usernameClick });
        }

        Array.from(element.querySelectorAll('.username')).forEach(function (usernameEl) {
            usernameEl.addEventListener('click', usernameClick);
            usernameEl.addEventListener('mouseenter', usernameEnter);
            usernameEl.addEventListener('mouseleave', usernameLeave);
            listeners.push(
                { target: usernameEl, type: 'click', handler: usernameClick },
                { target: usernameEl, type: 'mouseenter', handler: usernameEnter },
                { target: usernameEl, type: 'mouseleave', handler: usernameLeave }
            );
        });

        Array.from(element.querySelectorAll('.button')).forEach(function (buttonEl) {
            let handler = null;
            switch (buttonEl.classList[1]) {
                case 'edit':
                    handler = messageButtonEdit;
                    buttonEl.addEventListener('click', handler);
                    break;
                case 'delete':
                    handler = messageButtonDelete;
                    buttonEl.addEventListener('click', handler);
                    break;
                case 'report':
                    /* buttonEl.addEventListener('click', messageButtonReport); */
                    break;
                default:
                    console.log("Unable to find use for button.", buttonEl);
                    break;
            }
            if (handler) {
                listeners.push({ target: buttonEl, type: 'click', handler: handler });
            }
        });

        // Store listeners for cleanup
        eventListenerMap.set(element, listeners);
    }

    function messageRemoveEventListeners(element) {
        const listeners = eventListenerMap.get(element);
        if (listeners) {
            listeners.forEach(({ target, type, handler }) => {
                target.removeEventListener(type, handler);
            });
            eventListenerMap.delete(element);
        }
    }

    function messageButtonDelete() {
        let messageEl = this.closest(".chat-message");
        if (messageEl !== null) {
            let template = document.getElementById('tmp-chat-modal-delete').content.cloneNode(true);
            let modal = template.children[0];
            modal.id = "chat-modal-delete";

            // Create a lightweight copy without event listeners
            // Instead of cloning the entire element, just clone the visual parts
            let messagePreview = document.createElement('div');
            messagePreview.className = 'chat-message';
            messagePreview.innerHTML = messageEl.querySelector('.main-content').innerHTML;

            modal.querySelector('.modal-message').appendChild(messagePreview);

            const cancelHandler = function () {
                window.MicroModal.close(modal.id);
            };
            const deleteHandler = function () {
                messageSend(`/delete ${messageEl.dataset.id}`);
                window.MicroModal.close(modal.id);
            };

            modal.querySelector('.button.cancel').addEventListener('click', cancelHandler);
            modal.querySelector('.button.delete').addEventListener('click', deleteHandler);

            document.body.appendChild(modal);

            // https://micromodal.vercel.app/#configuration
            window.MicroModal.show(modal.id, {
                onClose: modal => {
                    let modalEl = document.getElementById(modal.id);
                    if (modalEl) {
                        // Clean up event listeners
                        let cancelBtn = modalEl.querySelector('.button.cancel');
                        let deleteBtn = modalEl.querySelector('.button.delete');
                        if (cancelBtn) cancelBtn.removeEventListener('click', cancelHandler);
                        if (deleteBtn) deleteBtn.removeEventListener('click', deleteHandler);
                        modalEl.remove();
                    }
                },
                openClass: 'is-open',
                disableScroll: true,
                disableFocus: false,
                awaitOpenAnimation: false,
                awaitCloseAnimation: false,
                debugMode: false
            });
        }
        else {
            console.log("Error: Cannot find chat message for delete button?");
        }
    }

    function messageButtonEdit() {
        let messageEl = this.closest(".chat-message");
        if (messageEl !== null) {
            messageEdit(messageEl);
        }
        else {
            console.log("Error: Cannot find chat message for delete button?");
        }
    }

    function messageDelete(message) {
        let el = document.getElementById(`chat-message-${message}`);
        let next = el.nextElementSibling;

        // Clean up event listeners before removing
        messageRemoveEventListeners(el);
        el.remove();
        messageSetHasParent(next);

        lastScrollPos = 0;
    }

    function messageEdit(messageEl) {
        messageEditReverse();

        messageEl.classList.add("chat-message--editing");

        let contentEl = messageEl.querySelector('.message');
        messageEl.originalMessage = contentEl.outerHTML;

        let formEl = document.getElementById("new-message-form").cloneNode(true);
        formEl.id = "edit-message-form";

        let inputEl = formEl.querySelector(".chat-input");
        inputEl.id = "edit-message-input";

        let submitEl = formEl.querySelector("button.submit");
        //submitEl.id = "edit-message-input";
        submitEl.remove();

        contentEl.replaceWith(formEl);

        const editValue = typeof messageEl.rawMessage === 'string' && messageEl.rawMessage.length
            ? messageEl.rawMessage
            : messageEl.querySelector('.message').textContent;

        inputEl.textContent = editValue;
        inputAddEventListeners(inputEl);
        inputEl.addEventListener('keydown', function (event) {
            switch (event.key) {
                case "Escape":
                    event.preventDefault();
                    messageEditReverse();
                    return false;

                case "Enter":
                    event.preventDefault();

                    messageSend("/edit " + JSON.stringify({
                        id: parseInt(messageEl.dataset.id, 10),
                        message: this.textContent,
                    }));
                    messageEditReverse();

                    return false;
            }
        });

        // Apparently, .focus() doesn't work on contenteditable=true until one frame after.
        inputFocusEnd(inputEl);
    }

    function messageEditReverse() {
        Array.from(document.querySelectorAll('.chat-message--editing')).forEach(function (el) {
            let contentEl = el.querySelector('.chat-form').outerHTML = el.originalMessage;
            el.classList.remove("chat-message--editing");
            lastScrollPos = 0;
            document.getElementById('new-message-input').focus({ preventScroll: true });
        });
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

        let id = null;
        let extantEl = null;
        let messagesEl = document.getElementById('chat-messages');
        let template = document.getElementById('tmp-chat-message').content.cloneNode(true);

        template.querySelector('.message').innerHTML = message.message;

        if (author) {
            id = parseInt(message.message_id, 10);
            extantEl = document.getElementById(`chat-message-${id}`);

            template.children[0].rawMessage = decodeHtmlEntities(message.message_raw);
            template.children[0].id = `chat-message-${id}`;
            template.children[0].dataset.id = id;
            template.children[0].dataset.author = author.id;
            template.children[0].dataset.timestamp = message.message_date;

            // Ignored poster?
            if (APP.user.ignored_users.includes(author.id)) {
                template.children[0].classList.add("chat-message--isIgnored");
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
                let avatarEl = template.querySelector('.avatar');
                avatarEl.setAttribute('src', author.avatar_url);
                avatarEl.setAttribute('loading', 'lazy');
                avatarEl.setAttribute('decoding', 'async');
            }
            else {
                template.querySelector('.avatar').remove();
            }

            // Add right-content details
            if (message.author.id != APP.user.id) {
                template.querySelector('.edit').remove();

                if (!APP.user.is_staff) {
                    template.querySelector('.delete').remove();
                }
            }
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

        let el = template.children[0];
        messageAddEventListeners(el);

        if (extantEl !== null) {
            extantEl.replaceWith(el);
        }
        else {
            el = messagesEl.appendChild(el);
        }

        messageSetHasParent(el);

        // Prune oldest messages with proper cleanup
        while (messagesEl.children.length > 200) {
            let oldMessage = messagesEl.children[0];

            // Clean up event listeners
            messageRemoveEventListeners(oldMessage);

            // Clear avatar src to help with memory cleanup
            let avatarEl = oldMessage.querySelector('.avatar');
            if (avatarEl) {
                avatarEl.removeAttribute('src');
            }

            oldMessage.remove();
            lastScrollPos = 0;
        }

        if (messagesEl.children.length > 0) {
            messagesEl.children[0].classList.remove("chat-message--hasParent");
        }

        // Scroll down.
        scrollToNew();

        return el;
    }

    function messageSend(message) {
        ws.send(message);
    }

    function messageSetHasParent(el) {
        let prev = el.previousElementSibling;

        if (prev !== null) {
            if (prev.dataset.author == el.dataset.author) {
                // Allow to break into new groups if too much time has passed.
                let timeLast = parseInt(prev.dataset.timestamp, 10);
                let timeNext = parseInt(el.dataset.timestamp, 10);
                if (timeNext - timeLast < 30) {
                    el.classList.add("chat-message--hasParent");
                    return true;
                }
            }
        }

        el.classList.remove("chat-message--hasParent");
        return false;
    }

    function messagesReceive(data) {
        let json = null;

        // Try to parse JSON data.
        try {
            json = JSON.parse(data);
        }
        // Not valid JSON, default
        catch (error) {
            messagePush({ message: data }, null);
            return;
        }

        if (json.hasOwnProperty('messages')) {
            json.messages.forEach(message => messagePush(message, message.author));
        }

        if (json.hasOwnProperty('delete')) {
            json.delete.forEach(message => messageDelete(message));
        }

        if (json.hasOwnProperty('users')) {
            Object.entries(json.users).forEach(user => {
                let [key, value] = user;
                userActivity(key, value);
            });
            userActivitySort();
        }
    }

    function messagesDelete() {
        let messagesEl = document.getElementById('chat-messages');
        while (messagesEl.firstChild) {
            // Clean up event listeners before removing
            if (messagesEl.firstChild.classList && messagesEl.firstChild.classList.contains('chat-message')) {
                messageRemoveEventListeners(messagesEl.firstChild);
            }
            messagesEl.removeChild(messagesEl.firstChild);
        }
    }

    function roomJoin(id) {
        if (Number.isInteger(id) && id > 0) {
            scrollEl.classList.remove('ScrollAnchored');
            scrollEl.classList.add('ScrollAnchorConsume');
            messagesDelete();
            userActivityDelete();
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

    function scheduleScrollCheck() {
        if (scrollAnimationFrame) {
            cancelAnimationFrame(scrollAnimationFrame);
        }
        scrollAnimationFrame = requestAnimationFrame(function tick() {
            scrollToNew();
            scrollAnimationFrame = requestAnimationFrame(tick);
        });
    }

    function userActivity(id, activity) {
        if (id == 0)
            return;

        let userEl = document.getElementById(`chat-activity-${id}`);

        if (activity !== false) {
            userActivityData[id] = activity;
            userActivityTouch(id);
        }
        else {
            delete userActivityData[id];

            if (userEl) {
                // Clean up avatar before removing
                let avatarEl = userEl.querySelector('.avatar');
                if (avatarEl) {
                    avatarEl.removeAttribute('src');
                    avatarEl.src = '';
                }
                userEl.remove();
            }
        }
    }

    function userActivityDelete() {
        let userEl = document.getElementById(`chat-activity`);
        while (userEl.firstChild) {
            // Clean up avatar references before removing
            let avatarEl = userEl.firstChild.querySelector('.avatar');
            if (avatarEl) {
                avatarEl.removeAttribute('src');
                avatarEl.src = '';
            }
            userEl.removeChild(userEl.firstChild);
        }
        // Clear the data object
        userActivityData = {};
    }

    function userActivityTouch(id) {
        if (userActivityData[id]) {
            let userEl = document.getElementById(`chat-activity-${id}`);
            userActivityData[id].last_activity = new Date;

            if (userEl) {
                // ???
            }
            else {
                let usersEl = document.getElementById('chat-activity');
                let newEl = document.getElementById('tmp-chat-user').content.cloneNode(true).children[0];

                newEl.id = `chat-activity-${id}`;
                newEl.dataset.username = userActivityData[id].username;
                newEl.last_activity = userActivityData[id].last_activity;

                let avEl = newEl.querySelector('.avatar');
                if (userActivityData[id].avatar_url) {
                    avEl.src = userActivityData[id].avatar_url;
                    avEl.alt = userActivityData[id].username;
                    avEl.setAttribute('loading', 'lazy');
                    avEl.setAttribute('decoding', 'async');
                }
                else {
                    avEl.remove();
                }

                let nameEl = newEl.querySelector('.user');
                nameEl.textContent = userActivityData[id].username;

                usersEl.appendChild(newEl);
            }
        }
    }

    function userActivitySort() {
        let usersEl = document.getElementById('chat-activity');
        let activityEls = usersEl.querySelectorAll(".activity");
        let time = (new Date).getTime();

        let sorted = Array.from(activityEls).sort((a, b) => {
            let ar = (a.last_activity.getTime() - time) <= 30000;
            let br = (b.last_activity.getTime() - time) <= 30000;

            if (ar == br) {
                return a.dataset.username.toLowerCase().localeCompare(b.dataset.username.toLowerCase());
            }
            else if (ar && !br) {
                return -1;
            }
            else {
                return 1;
            }
        })

        sorted.forEach(e => usersEl.appendChild(e));
    }

    function usernameClick(event) {
        // TODO: Replace with Dialog like Discord?
        let inputEl = document.getElementById('new-message-input')
        inputEl.textContent += `@${this.textContent}, `;

        inputFocusEnd(inputEl);

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
        // Clean up existing websocket if it exists
        if (ws !== null) {
            try {
                ws.close();
            } catch (e) {
                console.log("Error closing old websocket:", e);
            }
            ws = null;
        }

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
    scheduleScrollCheck();

    // Form
    document.getElementById('new-message-input').addEventListener('keydown', function (event) {
        switch (event.key) {
            case "Enter":
                event.preventDefault();

                messageSend(this.textContent);
                this.textContent = "";

                return false;

            case "ArrowUp":
                if (!this.innerHTML) {
                    event.preventDefault();

                    let messageEls = document.getElementById('chat-messages').querySelectorAll(`.chat-message[data-author='${APP.user.id}']`);
                    if (messageEls.length > 0) {
                        let messageEl = messageEls[messageEls.length - 1];
                        messageEdit(messageEl);
                    }

                    return false;
                }
        }
    });
    inputAddEventListeners(document.getElementById('new-message-input'));

    document.getElementById('new-message-submit').addEventListener('click', function (event) {
        event.preventDefault();
        let input = document.getElementById('new-message-input');

        messageSend(input.innerHTML);
        input.innerHTML = "";

        input.focus({ preventScroll: true });
        return false;
    });

    // Safely terminate websocket so server knows we're disconnecting.
    window.addEventListener('beforeunload', function () {
        if (ws && ws.readyState == WebSocket.OPEN) {
            ws.onclose = function () { };
            ws.close(1000, "Bye!");
        }

        // Cancel animation frame
        if (scrollAnimationFrame) {
            cancelAnimationFrame(scrollAnimationFrame);
        }
    });

    window.addEventListener('hashchange', roomJoinByHash, false);

    window.addEventListener('keydown', function (event) {
        switch (event.key) {
            case "Escape": messageEditReverse(); break;
        }
    });

    window.addEventListener('resize', function (event) {
        if (!scrollEl.classList.contains("ScrollAnchor")) {
            scrollEl.classList.add("ScrollAnchorConsume");
        }
        scrollToNew();
    });

    websocketConnect();
});
