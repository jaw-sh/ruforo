import * as ws from './ws';
import * as mentions from './mentions';
import { handleToolbarShortcut } from './toolbar';
import { messageEdit, messageEditReverse, messagePushPending } from './messages';

export function initInput(): void {
  const inputEl = document.getElementById('new-message-input')!;
  const submitEl = document.getElementById('new-message-submit')!;

  inputAddEventListeners(inputEl);

  inputEl.addEventListener('keydown', (e: KeyboardEvent) => {
    // Mention dropdown intercepts keys first
    if (mentions.handleKeydown(e)) return;

    // Toolbar shortcuts (Ctrl+B, etc.)
    if (handleToolbarShortcut(e)) return;

    switch (e.key) {
      case 'Enter':
        if (e.shiftKey) {
          // Shift+Enter: insert line break
          // Let the browser handle it naturally in contenteditable
          return;
        }
        e.preventDefault();
        submitMessage(inputEl);
        return;

      case 'ArrowUp': {
        // Edit last message if cursor is at start or input is empty
        const text = inputEl.textContent || '';
        if (text.length === 0 || isCursorAtStart(inputEl)) {
          e.preventDefault();
          const messageEls = document.getElementById('chat-messages')!.querySelectorAll(
            `.chat-message[data-author='${APP.user.id}']:not(.chat-message--whisper)`
          );
          if (messageEls.length > 0) {
            messageEdit(messageEls[messageEls.length - 1] as HTMLElement);
          }
          return;
        }
        break;
      }

      case 'Escape':
        messageEditReverse();
        break;
    }
  });

  // Auto-grow on input
  inputEl.addEventListener('input', () => {
    autoGrow(inputEl);
    mentions.handleInput(inputEl);
  });

  submitEl.addEventListener('click', (e: Event) => {
    e.preventDefault();
    submitMessage(inputEl);
    inputEl.focus({ preventScroll: true });
  });
}

export function inputAddEventListeners(el: HTMLElement): void {
  // Strip <br> and <div> elements that mobile browsers inject
  el.addEventListener('input', function (this: HTMLElement) {
    Array.from(this.querySelectorAll('br, div')).forEach((node) => {
      if (node.tagName === 'BR') {
        // Keep BRs that are from Shift+Enter
        return;
      } else if (node.tagName === 'DIV') {
        // Firefox wraps new lines in <div>; unwrap the text content
        node.replaceWith(document.createTextNode(node.textContent || ''));
      }
    });
  });

  // Text-only paste
  el.addEventListener('paste', function (this: HTMLElement, event: ClipboardEvent) {
    const text = event.clipboardData?.getData('text/plain');
    if (!text) return;

    const sel = window.getSelection();
    if (!sel || !sel.rangeCount) return;
    sel.deleteFromDocument();

    const range = sel.getRangeAt(0);
    const newNode = document.createTextNode(text);
    range.insertNode(newNode);
    range.setStart(this, range.endOffset);

    event.preventDefault();

    // Auto-grow after paste
    autoGrow(this);
  });
}

export function inputFocusEnd(el: HTMLElement): void {
  setTimeout(() => {
    const range = document.createRange();
    range.selectNodeContents(el);
    range.collapse(false);

    const sel = window.getSelection();
    if (sel) {
      sel.removeAllRanges();
      sel.addRange(range);
    }
    el.focus();
  }, 0);
}

/** When true, show a grayed-out local message until the server echoes it back. */
let showPendingMessages = false;

export function setShowPendingMessages(enabled: boolean): void {
  showPendingMessages = enabled;
}

function expandWhisperReply(text: string): string {
  if (!text.startsWith('/r ') && text !== '/r') return text;

  // Find the last whisper message in the DOM
  const whispers = document.querySelectorAll('.chat-message--whisper');
  if (whispers.length === 0) return text;

  const lastWhisper = whispers[whispers.length - 1] as HTMLElement;
  const partner = lastWhisper.dataset.whisperPartner;
  if (!partner) return text;

  const rest = text.length > 3 ? text.substring(3) : '';
  return `/w @${partner}, ${rest}`;
}

function submitMessage(inputEl: HTMLElement): void {
  let text = inputEl.textContent?.trim() || '';
  if (text.length === 0) return;

  // Expand /r to /w @LastPartner,
  text = expandWhisperReply(text);

  if (showPendingMessages) {
    const pending = ws.sendChat(text);
    messagePushPending(pending);
  } else {
    ws.send(text);
  }

  inputEl.textContent = '';
  autoGrow(inputEl);

  mentions.dismiss();
}

function autoGrow(el: HTMLElement): void {
  // Collapse to 0 so scrollHeight reflects actual content, not previous height
  el.style.height = '0';
  el.style.height = `${el.scrollHeight}px`;
}

function isCursorAtStart(el: HTMLElement): boolean {
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0) return false;

  const range = sel.getRangeAt(0);
  if (!range.collapsed) return false;

  // If cursor is at offset 0 of the first child (or the element itself)
  if (range.startOffset === 0) {
    let node: Node | null = range.startContainer;
    while (node && node !== el) {
      if (node.previousSibling) return false;
      node = node.parentNode;
    }
    return true;
  }

  return false;
}
