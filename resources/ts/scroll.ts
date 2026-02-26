/** Manages scroll anchoring — auto-scroll to bottom unless user scrolled up. */

let scrollEl: HTMLElement;
let lastScrollPos = 0;
let scrollAnimationFrame: number | null = null;
let stopScrollCheckFn: (() => void) | null = null;

export function initScroll(el: HTMLElement): void {
  scrollEl = el;
  scrollEl.addEventListener('scroll', onScroll);
  stopScrollCheckFn = scheduleScrollCheck();
}

export function destroyScroll(): void {
  if (stopScrollCheckFn) {
    stopScrollCheckFn();
    stopScrollCheckFn = null;
  }
}

export function scrollToNew(): void {
  if (!scrollEl.classList.contains('ScrollAnchored')) {
    scrollEl.scrollTo(0, scrollEl.scrollHeight);
  }
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

function scheduleScrollCheck(): () => void {
  if (scrollAnimationFrame) {
    cancelAnimationFrame(scrollAnimationFrame);
    scrollAnimationFrame = null;
  }

  let isRunning = true;

  function tick(): void {
    if (!isRunning) return;
    scrollToNew();
    scrollAnimationFrame = requestAnimationFrame(tick);
  }

  scrollAnimationFrame = requestAnimationFrame(tick);

  return () => {
    isRunning = false;
    if (scrollAnimationFrame) {
      cancelAnimationFrame(scrollAnimationFrame);
      scrollAnimationFrame = null;
    }
  };
}
