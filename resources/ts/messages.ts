import type { Author, SanitaryPost, PendingMessage } from './types';
import MicroModal from 'micromodal';
import * as ws from './ws';
import { getRoomPermissions } from './chat';
import { scrollToNew, resetLastScroll } from './scroll';

// 1x1 transparent GIF for avatar cleanup
const BLANK_GIF = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';

// Track event listeners for cleanup
const eventListenerMap = new WeakMap<HTMLElement, Array<{ target: EventTarget; type: string; handler: EventListener }>>();

// Reuse a DOM node to decode HTML entities back into plain text
const decodeHtmlEntities = (() => {
  const textarea = document.createElement('textarea');
  return (value: string): string => {
    if (typeof value !== 'string') {
      return '';
    }
    textarea.innerHTML = value;
    const decoded = textarea.value;
    textarea.value = '';
    return decoded;
  };
})();

let messageHoverEl: HTMLElement | null = null;
let userHover: number | null = null;

// Export these for use from input.ts
export { messageEdit, messageEditReverse };

// ---------------------------------------------------------------------------
// Input helpers (duplicated from chat.js — will be moved to input.ts later)
// ---------------------------------------------------------------------------

function inputAddEventListeners(el: HTMLElement): void {
  // Strip <br> and <div> elements that mobile browsers inject into
  // contenteditable during composition.
  el.addEventListener('input', function (this: HTMLElement) {
    Array.from(this.querySelectorAll('br, div')).forEach(function (node) {
      if (node.tagName === 'BR') {
        node.remove();
      } else if (node.tagName === 'DIV') {
        node.replaceWith(document.createTextNode(node.textContent ?? ''));
      }
    });
  });

  el.addEventListener('paste', function (this: HTMLElement, event: ClipboardEvent) {
    const text = event.clipboardData?.getData('text/plain') ?? '';

    const sel = window.getSelection();
    if (!sel || !sel.rangeCount) {
      return false;
    }
    sel.deleteFromDocument();

    const range = sel.getRangeAt(0);
    const newNode = document.createTextNode(text);
    range.insertNode(newNode);
    range.setStart(this, range.endOffset);

    event.preventDefault();
    return false;
  });
}

function inputFocusEnd(el: HTMLElement): void {
  setTimeout(function () {
    const range = document.createRange();
    range.setStart(el, el.childElementCount + 1);
    range.setEnd(el, el.childElementCount + 1);
    range.collapse(false);

    const sel = window.getSelection();
    if (sel) {
      sel.removeAllRanges();
      sel.addRange(range);
    }
    el.focus();
  }, 0);
}

// ---------------------------------------------------------------------------
// Event listener management
// ---------------------------------------------------------------------------

export function messageAddEventListeners(element: HTMLElement): void {
  const listeners: Array<{ target: EventTarget; type: string; handler: EventListener }> = [];

  if (Object.keys(element.dataset).indexOf('author') > -1) {
    element.addEventListener('mouseenter', messageMouseEnter as EventListener);
    element.addEventListener('mouseleave', messageMouseLeave as EventListener);
    listeners.push(
      { target: element, type: 'mouseenter', handler: messageMouseEnter as EventListener },
      { target: element, type: 'mouseleave', handler: messageMouseLeave as EventListener },
    );
  }

  const authorEl = element.querySelector('.author') as HTMLElement | null;
  if (authorEl !== null) {
    authorEl.addEventListener('click', usernameClick as EventListener);
    listeners.push({ target: authorEl, type: 'click', handler: usernameClick as EventListener });
  }

  Array.from(element.querySelectorAll('.username')).forEach(function (usernameEl) {
    usernameEl.addEventListener('click', usernameClick as EventListener);
    usernameEl.addEventListener('mouseenter', usernameEnter as EventListener);
    usernameEl.addEventListener('mouseleave', usernameLeave as EventListener);
    listeners.push(
      { target: usernameEl, type: 'click', handler: usernameClick as EventListener },
      { target: usernameEl, type: 'mouseenter', handler: usernameEnter as EventListener },
      { target: usernameEl, type: 'mouseleave', handler: usernameLeave as EventListener },
    );
  });

  Array.from(element.querySelectorAll('.button')).forEach(function (buttonEl) {
    let handler: EventListener | null = null;
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
        console.log('Unable to find use for button.', buttonEl);
        break;
    }
    if (handler) {
      listeners.push({ target: buttonEl, type: 'click', handler: handler });
    }
  });

  // [ditto] button click handler
  Array.from(element.querySelectorAll('.tagDitto')).forEach(function (dittoEl) {
    const dittoHandler: EventListener = function () {
      const inputEl = document.getElementById('new-message-input');
      if (inputEl) {
        inputEl.textContent += dittoEl.textContent ?? '';
        inputFocusEnd(inputEl);
      }
    };
    dittoEl.addEventListener('click', dittoHandler);
    listeners.push({ target: dittoEl, type: 'click', handler: dittoHandler });
  });

  // Store listeners for cleanup
  eventListenerMap.set(element, listeners);
}

export function messageRemoveEventListeners(element: HTMLElement): void {
  const listeners = eventListenerMap.get(element);
  if (listeners) {
    listeners.forEach(({ target, type, handler }) => {
      target.removeEventListener(type, handler);
    });
    eventListenerMap.delete(element);
  }
}

// ---------------------------------------------------------------------------
// Delete confirmation modal
// ---------------------------------------------------------------------------

function messageButtonDelete(this: HTMLElement): void {
  const messageEl = this.closest('.chat-message') as HTMLElement | null;
  if (messageEl !== null) {
    const template = (document.getElementById('tmp-chat-modal-delete') as HTMLTemplateElement).content.cloneNode(true) as DocumentFragment;
    const modal = template.children[0] as HTMLElement;
    modal.id = 'chat-modal-delete';

    // Create a lightweight text-only copy to avoid image memory issues
    const messagePreview = document.createElement('div');
    messagePreview.className = 'chat-message';

    // Clone the main content but strip images to prevent memory retention
    const mainContent = messageEl.querySelector('.main-content');
    if (mainContent) {
      const clonedContent = mainContent.cloneNode(true) as HTMLElement;
      // Remove all images from the preview to prevent memory leaks
      Array.from(clonedContent.querySelectorAll('img')).forEach((img) => {
        const placeholder = document.createElement('span');
        placeholder.textContent = '[image]';
        img.replaceWith(placeholder);
      });
      messagePreview.innerHTML = clonedContent.innerHTML;
    }

    modal.querySelector('.modal-message')!.appendChild(messagePreview);

    const cancelHandler = function () {
      MicroModal.close(modal.id);
    };
    const deleteHandler = function () {
      ws.send(`/delete ${messageEl.dataset.id}`);
      MicroModal.close(modal.id);
    };

    modal.querySelector('.button.cancel')!.addEventListener('click', cancelHandler);
    modal.querySelector('.button.delete')!.addEventListener('click', deleteHandler);

    document.body.appendChild(modal);

    // https://micromodal.vercel.app/#configuration
    MicroModal.show(modal.id, {
      onClose: (closedModal: { id: string }) => {
        const modalEl = document.getElementById(closedModal.id);
        if (modalEl) {
          // Clean up event listeners
          const cancelBtn = modalEl.querySelector('.button.cancel');
          const deleteBtn = modalEl.querySelector('.button.delete');
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
      debugMode: false,
    });
  } else {
    console.log('Error: Cannot find chat message for delete button?');
  }
}

// ---------------------------------------------------------------------------
// Edit button
// ---------------------------------------------------------------------------

function messageButtonEdit(this: HTMLElement): void {
  const messageEl = this.closest('.chat-message') as HTMLElement | null;
  if (messageEl !== null) {
    messageEdit(messageEl);
  } else {
    console.log('Error: Cannot find chat message for delete button?');
  }
}

// ---------------------------------------------------------------------------
// Avatar cleanup
// ---------------------------------------------------------------------------

export function cleanupAvatarImage(avatarEl: HTMLImageElement | null): void {
  if (!avatarEl) return;

  // Force the browser to release the decoded bitmap by replacing with tiny image
  avatarEl.src = BLANK_GIF;

  // Remove other attributes
  avatarEl.removeAttribute('srcset');
  avatarEl.removeAttribute('loading');
  avatarEl.removeAttribute('decoding');
  avatarEl.removeAttribute('alt');

  // Now remove from DOM
  avatarEl.remove();
}

// ---------------------------------------------------------------------------
// Message deletion
// ---------------------------------------------------------------------------

export function messageDelete(messageUuid: string): void {
  const el = document.getElementById(`chat-message-${messageUuid}`);
  if (!el) return;

  const next = el.nextElementSibling as HTMLElement | null;

  // Release any pending message DOM references
  ws.releasePendingElement(el);

  // Clear hover reference if it points at this element
  if (messageHoverEl === el) {
    messageHoverEl = null;
  }

  // Clean up event listeners before removing
  messageRemoveEventListeners(el as HTMLElement);

  // Remove avatar from DOM entirely to free decoded bitmap memory
  cleanupAvatarImage(el.querySelector('.avatar') as HTMLImageElement | null);

  el.remove();
  if (next) {
    messageSetHasParent(next);
  }

  resetLastScroll();
}

// ---------------------------------------------------------------------------
// Message editing
// ---------------------------------------------------------------------------

function messageEdit(messageEl: HTMLElement): void {
  messageEditReverse();

  messageEl.classList.add('chat-message--editing');

  const contentEl = messageEl.querySelector('.message') as HTMLElement;

  const editValue = messageEl.dataset.raw
    ? decodeHtmlEntities(messageEl.dataset.raw)
    : contentEl.textContent;

  // Hide the original message (keep it in DOM for safe reversal)
  contentEl.style.display = 'none';

  const formEl = document.getElementById('new-message-form')!.cloneNode(true) as HTMLElement;
  formEl.id = 'edit-message-form';

  const inputEl = formEl.querySelector('.chat-input') as HTMLElement;
  inputEl.id = 'edit-message-input';

  const submitEl = formEl.querySelector('button.submit');
  if (submitEl) {
    submitEl.remove();
  }

  // Insert form after the hidden message
  contentEl.after(formEl);

  inputEl.textContent = editValue;
  inputAddEventListeners(inputEl);
  inputEl.addEventListener('keydown', function (this: HTMLElement, event: KeyboardEvent) {
    switch (event.key) {
      case 'Escape':
        event.preventDefault();
        messageEditReverse();
        return false;

      case 'Enter':
        // Shift+Enter inserts a newline — do not submit
        if (event.shiftKey) {
          return;
        }
        event.preventDefault();

        ws.send('/edit ' + JSON.stringify({
          uuid: messageEl.dataset.id,
          message: this.textContent,
        }));
        messageEditReverse();

        return false;
    }
  });

  // Apparently, .focus() doesn't work on contenteditable=true until one frame after.
  inputFocusEnd(inputEl);

  // Scroll the edit form into view (especially important for bottom messages)
  inputEl.scrollIntoView({ block: 'nearest' });
}

function messageEditReverse(): void {
  Array.from(document.querySelectorAll('.chat-message--editing')).forEach(function (el) {
    // Remove the edit form
    const formEl = el.querySelector('#edit-message-form');
    if (formEl) {
      formEl.remove();
    }
    // Restore the hidden message
    const contentEl = el.querySelector('.message') as HTMLElement | null;
    if (contentEl) {
      contentEl.style.display = '';
    }
    el.classList.remove('chat-message--editing');
    scrollToNew();
    const newInput = document.getElementById('new-message-input');
    if (newInput) {
      newInput.focus({ preventScroll: true });
    }
  });
}

// ---------------------------------------------------------------------------
// Mouse hover handlers
// ---------------------------------------------------------------------------

function messageMouseEnter(this: HTMLElement, _event: MouseEvent): void {
  const author = parseInt(this.dataset.author!, 10);

  // Are we already hovering over something?
  if (messageHoverEl !== null) {
    // Is it the same message?
    if (this === messageHoverEl) {
      return;
    }

    // Is it by the same author?
    if (author === parseInt(messageHoverEl.dataset.author!, 10)) {
      return;
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

function messageMouseLeave(this: HTMLElement, _event: MouseEvent): void {
  if (messageHoverEl !== null && messageHoverEl === this) {
    messageHoverEl = null;
    Array.from(document.querySelectorAll('.chat-message--highlightAuthor')).forEach(function (el) {
      el.classList.remove('chat-message--highlightAuthor');
    });
  }
}

// ---------------------------------------------------------------------------
// Username interaction handlers
// ---------------------------------------------------------------------------

function usernameClick(this: HTMLElement, event: Event): void {
  const inputEl = document.getElementById('new-message-input');
  if (inputEl) {
    inputEl.textContent += `@${this.textContent} `;
    inputFocusEnd(inputEl);
  }

  event.preventDefault();
}

function usernameEnter(this: HTMLElement, _event: Event): void {
  const id = parseInt(this.dataset.id!, 10);

  if (userHover === id) {
    return;
  }

  userHover = id;

  Array.from(document.querySelectorAll('.chat-message--highlightUser')).forEach(function (el) {
    el.classList.remove('chat-message--highlightUser');
  });
  Array.from(document.querySelectorAll(`[data-author='${id}']`)).forEach(function (el) {
    el.classList.add('chat-message--highlightUser');
  });
}

function usernameLeave(this: HTMLElement, _event: Event): void {
  const id = parseInt(this.dataset.id!, 10);

  if (userHover === id) {
    userHover = null;
    Array.from(document.querySelectorAll('.chat-message--highlightUser')).forEach(function (el) {
      el.classList.remove('chat-message--highlightUser');
    });
  }
}

// ---------------------------------------------------------------------------
// Message grouping (hasParent)
// ---------------------------------------------------------------------------

export function messageSetHasParent(el: HTMLElement): boolean {
  const prev = el.previousElementSibling as HTMLElement | null;

  if (prev !== null) {
    if (prev.dataset.author === el.dataset.author) {
      // Allow to break into new groups if too much time has passed.
      const timeLast = parseInt(prev.dataset.timestamp!, 10);
      const timeNext = parseInt(el.dataset.timestamp!, 10);
      if (timeNext - timeLast < 30) {
        el.classList.add('chat-message--hasParent');
        return true;
      }
    }
  }

  el.classList.remove('chat-message--hasParent');
  return false;
}

// ---------------------------------------------------------------------------
// Main message push
// ---------------------------------------------------------------------------

export function messagePush(message: SanitaryPost | { message: string }, author?: Author | null): HTMLElement {
  // Normalize: if given a plain string wrap it
  if (typeof message === 'string') {
    message = { message: message as string };
  }

  let extantEl: HTMLElement | null = null;
  const messagesEl = document.getElementById('chat-messages')!;
  const template = (document.getElementById('tmp-chat-message') as HTMLTemplateElement).content.cloneNode(true) as DocumentFragment;

  template.querySelector('.message')!.innerHTML = (message as SanitaryPost).message;

  if (author) {
    const msg = message as SanitaryPost;
    const uuid = msg.message_uuid;
    extantEl = document.getElementById(`chat-message-${uuid}`);

    // If this is our own message echoed back, resolve the oldest pending
    // message (FIFO — server broadcasts in send order).
    if (msg.author.id === APP.user.id) {
      const pending = ws.resolveNextPending();
      if (pending?.element) {
        extantEl = pending.element;
      }
    }

    const rootEl = template.children[0] as HTMLElement;
    rootEl.dataset.raw = msg.message_raw;
    rootEl.id = `chat-message-${uuid}`;
    rootEl.dataset.id = uuid;
    rootEl.dataset.author = String(author.id);
    rootEl.dataset.timestamp = String(msg.message_date);

    // Ignored poster?
    if (APP.user.ignored_users.includes(author.id)) {
      rootEl.classList.add('chat-message--isIgnored');
    }

    // Add meta details
    const authorEl = template.querySelector('.author') as HTMLElement;
    authorEl.innerHTML = author.username;
    authorEl.dataset.id = String(author.id);

    Array.from(template.querySelectorAll('.timestamp')).forEach(function (el) {
      const time = new Date(msg.message_date * 1000);
      const hours = time.getHours();
      const minutes = String(time.getMinutes()).padStart(2, '0');

      el.setAttribute('datetime', String(msg.message_date));

      if (el.classList.contains('relative')) {
        const dayThen = new Date(msg.message_date * 1000).setHours(0, 0, 0, 0);
        const dayNow = new Date().setHours(0, 0, 0, 0);

        // Same day, only show clock
        if (dayThen === dayNow) {
          el.innerHTML = time.toLocaleTimeString();
        }
        // Different days, show date too.
        else {
          el.innerHTML = time.toLocaleDateString() + ' ' + time.toLocaleTimeString();
        }
      } else {
        el.innerHTML = (hours % 12) + ':' + minutes + ' ' + (hours >= 12 ? 'PM' : 'AM');
      }
    });

    // Add left-content details
    if (author.avatar_url.length > 0) {
      const avatarEl = template.querySelector('.avatar') as HTMLImageElement;
      avatarEl.setAttribute('src', author.avatar_url);
      avatarEl.setAttribute('loading', 'lazy');
      avatarEl.setAttribute('decoding', 'async');
    } else {
      template.querySelector('.avatar')?.remove();
    }

    // Add right-content details based on room permissions
    const perms = getRoomPermissions();
    const isOwn = msg.author.id === APP.user.id;

    if (!(isOwn ? perms.can_edit_own : perms.can_edit_other)) {
      template.querySelector('.edit')?.remove();
    }

    if (!(isOwn ? perms.can_delete_own : perms.can_delete_other)) {
      template.querySelector('.delete')?.remove();
    }

    if (isOwn || !perms.can_report) {
      template.querySelector('.report')?.remove();
    } else {
      const reportEl = template.querySelector('.report');
      if (reportEl) {
        reportEl.setAttribute('href', `/chat/messages/${uuid}/report`);
        reportEl.setAttribute('target', '_blank');
        reportEl.setAttribute('rel', 'noopener');
      }
    }
  } else {
    const rootEl = template.children[0] as HTMLElement;
    rootEl.classList.add('chat-message--systemMsg');
    template.querySelector('.meta')?.remove();
    template.querySelector('.left-content')?.remove();
  }

  // Force set URLs to target new tab.
  Array.from(template.querySelectorAll('.bbcode-url')).forEach(function (el) {
    (el as HTMLAnchorElement).target = '_blank';
  });

  // Check tagging.
  if ((message as SanitaryPost).message.includes(`@${APP.user.username}`)) {
    (template.children[0] as HTMLElement).classList.add('chat-message--highlightYou');
  }

  let el = template.children[0] as HTMLElement;
  messageAddEventListeners(el);

  if (extantEl !== null) {
    // Clean up old element before replacing
    messageRemoveEventListeners(extantEl);
    cleanupAvatarImage(extantEl.querySelector('.avatar') as HTMLImageElement | null);
    extantEl.replaceWith(el);
  } else {
    el = messagesEl.appendChild(el) as HTMLElement;
  }

  messageSetHasParent(el);

  // Prune oldest messages with proper cleanup
  pruneMessages(messagesEl);

  if (messagesEl.children.length > 0) {
    (messagesEl.children[0] as HTMLElement).classList.remove('chat-message--hasParent');
  }

  // Scroll down.
  scrollToNew();

  return el;
}

// ---------------------------------------------------------------------------
// Message pruning (shared between messagePush and messagePushPending)
// ---------------------------------------------------------------------------

function pruneMessages(messagesEl: HTMLElement): void {
  while (messagesEl.children.length > 200) {
    const oldMessage = messagesEl.children[0] as HTMLElement;

    // Release pending DOM references pointing at this element
    ws.releasePendingElement(oldMessage);

    // Clear messageHoverEl if it points at this element
    if (messageHoverEl === oldMessage) {
      messageHoverEl = null;
    }

    // Clean up event listeners
    messageRemoveEventListeners(oldMessage);

    // Remove avatar from DOM entirely to free decoded bitmap memory
    cleanupAvatarImage(oldMessage.querySelector('.avatar') as HTMLImageElement | null);

    oldMessage.remove();
    resetLastScroll();
  }
}

// ---------------------------------------------------------------------------
// Pending message support
// ---------------------------------------------------------------------------

export function messagePushPending(pending: PendingMessage): HTMLElement {
  const messagesEl = document.getElementById('chat-messages')!;
  const template = (document.getElementById('tmp-chat-message') as HTMLTemplateElement).content.cloneNode(true) as DocumentFragment;

  const rootEl = template.children[0] as HTMLElement;
  rootEl.classList.add('chat-message--pending');
  rootEl.dataset.id = 'pending';
  rootEl.dataset.author = String(APP.user.id);
  rootEl.dataset.timestamp = String(pending.timestamp);

  // Set message content as plain text (not yet rendered by server)
  const messageContentEl = template.querySelector('.message') as HTMLElement;
  messageContentEl.textContent = pending.text;

  // Add meta details using current user
  const authorEl = template.querySelector('.author') as HTMLElement;
  if (authorEl) {
    authorEl.innerHTML = APP.user.username;
    authorEl.dataset.id = String(APP.user.id);
  }

  // Set timestamps
  Array.from(template.querySelectorAll('.timestamp')).forEach(function (el) {
    const time = new Date(pending.timestamp * 1000);
    const hours = time.getHours();
    const minutes = String(time.getMinutes()).padStart(2, '0');

    el.setAttribute('datetime', String(pending.timestamp));

    if (el.classList.contains('relative')) {
      el.innerHTML = time.toLocaleTimeString();
    } else {
      el.innerHTML = (hours % 12) + ':' + minutes + ' ' + (hours >= 12 ? 'PM' : 'AM');
    }
  });

  // Avatar
  if (APP.user.avatar_url.length > 0) {
    const avatarEl = template.querySelector('.avatar') as HTMLImageElement;
    avatarEl.setAttribute('src', APP.user.avatar_url);
    avatarEl.setAttribute('loading', 'lazy');
    avatarEl.setAttribute('decoding', 'async');
  } else {
    template.querySelector('.avatar')?.remove();
  }

  // Pending messages are ours, so keep edit button but remove report
  // Actually, hide all buttons for pending messages since they don't have a real ID yet
  template.querySelector('.edit')?.remove();
  template.querySelector('.delete')?.remove();
  template.querySelector('.report')?.remove();

  // Force set URLs to target new tab
  Array.from(template.querySelectorAll('.bbcode-url')).forEach(function (el) {
    (el as HTMLAnchorElement).target = '_blank';
  });

  const el = template.children[0] as HTMLElement;
  messageAddEventListeners(el);
  messagesEl.appendChild(el);

  messageSetHasParent(el);

  // Store reference on pending object so ws.ts can track it
  pending.element = el;

  // Prune oldest messages with proper cleanup
  pruneMessages(messagesEl);

  if (messagesEl.children.length > 0) {
    (messagesEl.children[0] as HTMLElement).classList.remove('chat-message--hasParent');
  }

  // Scroll down.
  scrollToNew();

  return el;
}

// ---------------------------------------------------------------------------
// Clear all messages (on room change)
// ---------------------------------------------------------------------------

export function messagesDelete(): void {
  // Clear module-level references to prevent retaining detached DOM trees
  messageHoverEl = null;

  const messagesEl = document.getElementById('chat-messages')!;
  while (messagesEl.firstChild) {
    const child = messagesEl.firstChild as HTMLElement;

    // Clean up event listeners before removing
    if (child.classList && child.classList.contains('chat-message')) {
      ws.releasePendingElement(child);
      messageRemoveEventListeners(child);

      // Remove avatar from DOM entirely to free decoded bitmap memory
      cleanupAvatarImage(child.querySelector('.avatar') as HTMLImageElement | null);

    }

    messagesEl.removeChild(child);
  }
}
