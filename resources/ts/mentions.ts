import { getActiveUsers } from './users';

let dropdownEl: HTMLElement | null = null;
let activeIndex = 0;
let mentionQuery = '';
let mentionStartOffset = 0;
let mentionAnchorNode: Node | null = null;

export function initMentions(): void {
  // No setup needed — triggered from input handler
}

export function handleInput(inputEl: HTMLElement): void {
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0) {
    dismiss();
    return;
  }

  const range = sel.getRangeAt(0);
  const node = range.startContainer;

  if (node.nodeType !== Node.TEXT_NODE) {
    dismiss();
    return;
  }

  const text = node.textContent || '';
  const cursorPos = range.startOffset;

  // Find the @ before cursor
  const beforeCursor = text.slice(0, cursorPos);
  const atIndex = beforeCursor.lastIndexOf('@');

  if (atIndex === -1) {
    dismiss();
    return;
  }

  // Make sure @ is at start or after a space
  if (atIndex > 0 && beforeCursor[atIndex - 1] !== ' ' && beforeCursor[atIndex - 1] !== '\n') {
    dismiss();
    return;
  }

  const query = beforeCursor.slice(atIndex + 1);

  // Don't show if there's a space in the query (mention is complete)
  if (query.includes(' ') || query.includes(',')) {
    dismiss();
    return;
  }

  mentionQuery = query.toLowerCase();
  mentionStartOffset = atIndex;
  mentionAnchorNode = node;

  showDropdown(inputEl);
}

export function handleKeydown(e: KeyboardEvent): boolean {
  if (!dropdownEl) return false;

  switch (e.key) {
    case 'Tab':
    case 'Enter':
      e.preventDefault();
      selectCurrent();
      return true;
    case 'ArrowDown':
      e.preventDefault();
      activeIndex = Math.min(activeIndex + 1, getFilteredItems().length - 1);
      updateActive();
      return true;
    case 'ArrowUp':
      e.preventDefault();
      activeIndex = Math.max(activeIndex - 1, 0);
      updateActive();
      return true;
    case 'Escape':
      e.preventDefault();
      dismiss();
      return true;
  }

  return false;
}

export function isActive(): boolean {
  return dropdownEl !== null;
}

export function dismiss(): void {
  if (dropdownEl) {
    // Clean up avatar images to release decoded bitmaps
    dropdownEl.querySelectorAll('img').forEach((img) => {
      (img as HTMLImageElement).src = '';
      img.remove();
    });
    dropdownEl.remove();
    dropdownEl = null;
  }
  activeIndex = 0;
  mentionQuery = '';
  mentionAnchorNode = null;
}

function getFilteredItems(): Array<{ id: string; username: string; avatar_url: string }> {
  const users = getActiveUsers();
  const items: Array<{ id: string; username: string; avatar_url: string }> = [];

  for (const [id, user] of Object.entries(users)) {
    if (id === '0' || id === String(APP.user.id)) continue;
    if (mentionQuery === '' || user.username.toLowerCase().startsWith(mentionQuery)) {
      items.push({ id, username: user.username, avatar_url: user.avatar_url });
    }
  }

  items.sort((a, b) => a.username.toLowerCase().localeCompare(b.username.toLowerCase()));
  return items.slice(0, 10); // Max 10 suggestions
}

function showDropdown(inputEl: HTMLElement): void {
  const items = getFilteredItems();

  if (items.length === 0) {
    dismiss();
    return;
  }

  if (!dropdownEl) {
    dropdownEl = document.createElement('div');
    dropdownEl.className = 'mention-dropdown';
    document.body.appendChild(dropdownEl);
  }

  // Position relative to input
  const inputRect = inputEl.getBoundingClientRect();
  dropdownEl.style.left = `${inputRect.left}px`;
  dropdownEl.style.bottom = `${window.innerHeight - inputRect.top + 4}px`;
  dropdownEl.style.position = 'fixed';

  // Render items
  activeIndex = Math.min(activeIndex, items.length - 1);

  dropdownEl.innerHTML = items.map((item, i) => {
    const avatarHtml = item.avatar_url
      ? `<img class="mention-avatar" src="${item.avatar_url}" loading="lazy" />`
      : '';
    return `<div class="mention-item${i === activeIndex ? ' active' : ''}" data-username="${item.username}">${avatarHtml}<span class="mention-name">${item.username}</span></div>`;
  }).join('');

  // Click handlers
  dropdownEl.querySelectorAll('.mention-item').forEach((el) => {
    el.addEventListener('mousedown', (e) => {
      e.preventDefault(); // Prevent input blur
      const username = (el as HTMLElement).dataset.username;
      if (username) insertMention(username);
    });
  });
}

function updateActive(): void {
  if (!dropdownEl) return;
  dropdownEl.querySelectorAll('.mention-item').forEach((el, i) => {
    el.classList.toggle('active', i === activeIndex);
  });
}

function selectCurrent(): void {
  const items = getFilteredItems();
  if (items[activeIndex]) {
    insertMention(items[activeIndex].username);
  }
}

function insertMention(username: string): void {
  if (!mentionAnchorNode || mentionAnchorNode.nodeType !== Node.TEXT_NODE) {
    dismiss();
    return;
  }

  const text = mentionAnchorNode.textContent || '';
  const before = text.slice(0, mentionStartOffset);
  const after = text.slice(mentionStartOffset + 1 + mentionQuery.length); // +1 for @
  const replacement = `@${username}, `;

  mentionAnchorNode.textContent = before + replacement + after;

  // Place cursor after the inserted mention
  const sel = window.getSelection();
  if (sel) {
    const range = document.createRange();
    const offset = before.length + replacement.length;
    range.setStart(mentionAnchorNode, offset);
    range.setEnd(mentionAnchorNode, offset);
    sel.removeAllRanges();
    sel.addRange(range);
  }

  dismiss();
}
