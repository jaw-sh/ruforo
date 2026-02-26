const TAG_MAP: Record<string, string> = {
  b: 'b',
  i: 'i',
  u: 'u',
  s: 's',
  code: 'code',
  url: 'url',
  img: 'img',
};

export function initToolbar(): void {
  // Attach click handlers to all .toolbar-btn elements
  // Each button has data-tag attribute
  document.querySelectorAll('.toolbar-btn').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.preventDefault();
      const tag = (btn as HTMLElement).dataset.tag;
      if (tag) wrapSelection(tag);
    });
  });
}

export function wrapSelection(tag: string): void {
  const input = document.getElementById('new-message-input');
  if (!input) return;

  input.focus();
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0) return;

  const range = sel.getRangeAt(0);

  // Check if selection is within our input
  if (!input.contains(range.commonAncestorContainer)) {
    // No selection in input — insert empty tags with cursor between
    const textNode = document.createTextNode(`[${tag}][/${tag}]`);
    // Append at end of input
    input.appendChild(textNode);
    // Place cursor between tags
    const newRange = document.createRange();
    const offset = `[${tag}]`.length;
    newRange.setStart(textNode, offset);
    newRange.setEnd(textNode, offset);
    sel.removeAllRanges();
    sel.addRange(newRange);
    return;
  }

  const selectedText = range.toString();

  if (selectedText.length > 0) {
    // Wrap selected text
    range.deleteContents();
    const wrapped = document.createTextNode(`[${tag}]${selectedText}[/${tag}]`);
    range.insertNode(wrapped);
    // Place cursor after the closing tag
    const newRange = document.createRange();
    newRange.setStartAfter(wrapped);
    newRange.setEndAfter(wrapped);
    sel.removeAllRanges();
    sel.addRange(newRange);
  } else {
    // No selection — insert empty tags
    const wrapped = document.createTextNode(`[${tag}][/${tag}]`);
    range.insertNode(wrapped);
    // Place cursor between tags
    const newRange = document.createRange();
    const offset = `[${tag}]`.length;
    newRange.setStart(wrapped, offset);
    newRange.setEnd(wrapped, offset);
    sel.removeAllRanges();
    sel.addRange(newRange);
  }
}

// Handle keyboard shortcuts: Ctrl+B, Ctrl+I, Ctrl+U, Ctrl+S
export function handleToolbarShortcut(e: KeyboardEvent): boolean {
  if (!e.ctrlKey && !e.metaKey) return false;

  let tag: string | null = null;
  switch (e.key.toLowerCase()) {
    case 'b': tag = 'b'; break;
    case 'i': tag = 'i'; break;
    case 'u': tag = 'u'; break;
    case 's': tag = 's'; break;
    default: return false;
  }

  e.preventDefault();
  wrapSelection(tag);
  return true;
}
