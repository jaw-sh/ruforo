/** Manages scroll anchoring — auto-scroll to bottom unless user scrolled up. */

let scrollEl: HTMLElement;
let lastScrollPos = 0;
let pendingScroll = false;

export function initScroll(el: HTMLElement): void {
  scrollEl = el;
  scrollEl.addEventListener('scroll', onScroll);
}

export function destroyScroll(): void {
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
