/** Manages scroll anchoring — auto-scroll to bottom unless user scrolled up. */

let scrollEl: HTMLElement;
let anchorBtn: HTMLElement | null = null;
let lastScrollPos = 0;
let pendingScroll = false;
let resizeObserver: ResizeObserver | null = null;

export function initScroll(el: HTMLElement): void {
  scrollEl = el;
  scrollEl.addEventListener('scroll', onScroll);

  // Wire up the "scroll to bottom" button
  anchorBtn = el.querySelector('#scroll-anchor-btn');
  if (anchorBtn) {
    anchorBtn.addEventListener('click', () => {
      resetScrollAnchor();
      scrollEl.scrollTo(0, scrollEl.scrollHeight);
    });
  }

  // Watch the content container for size changes (e.g. images loading in)
  // so we re-anchor the scroll when content height grows.
  const contentEl = el.querySelector('#chat-messages');
  if (contentEl) {
    resizeObserver = new ResizeObserver(() => {
      scrollToNew();
    });
    resizeObserver.observe(contentEl);
  }
}

export function destroyScroll(): void {
  if (resizeObserver) {
    resizeObserver.disconnect();
    resizeObserver = null;
  }
  if (scrollEl) {
    scrollEl.removeEventListener('scroll', onScroll);
  }
}

export function scrollToNew(): void {
  if (pendingScroll) return;
  pendingScroll = true;

  requestAnimationFrame(() => {
    pendingScroll = false;
    if (!scrollEl.classList.contains('ScrollAnchored')) {
      scrollEl.scrollTo(0, scrollEl.scrollHeight);
    }
  });
}

export function resetScrollAnchor(): void {
  scrollEl.classList.remove('ScrollAnchored');
  scrollEl.classList.add('ScrollAnchorConsume');
  lastScrollPos = scrollEl.scrollTop;
}

export function resetLastScroll(): void {
  lastScrollPos = 0;
}

function onScroll(this: HTMLElement): void {
  const clampHeight = 64;

  if (lastScrollPos > this.scrollTop) {
    if (!this.classList.contains('ScrollAnchorConsume')) {
      this.classList.add('ScrollAnchored');
    } else {
      this.classList.remove('ScrollAnchorConsume');
    }
  } else if (this.offsetHeight + this.scrollTop >= this.scrollHeight - clampHeight) {
    this.classList.remove('ScrollAnchored');
  }

  lastScrollPos = this.scrollTop;
}
