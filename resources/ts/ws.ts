/**
 * WebSocket manager with reconnect logic and pending message queue.
 * Handles connection lifecycle and message delivery resilience.
 */

import type { PendingMessage, ServerPayload } from './types';

type MessageHandler = (data: ServerPayload) => void;
type SystemHandler = (text: string) => void;
type RawHandler = (text: string) => void;

let ws: WebSocket | null = null;
let onMessage: MessageHandler | null = null;
let onSystem: SystemHandler | null = null;
let onRaw: RawHandler | null = null;
let onConnected: (() => void) | null = null;
let onDisconnected: (() => void) | null = null;

const pendingMessages: PendingMessage[] = [];
let currentRoom: number | null = null;

const ECHO_TIMEOUT_MS = 30_000;

export function configure(handlers: {
  onMessage: MessageHandler;
  onSystem: SystemHandler;
  onRaw: RawHandler;
  onConnected: () => void;
  onDisconnected: () => void;
}): void {
  onMessage = handlers.onMessage;
  onSystem = handlers.onSystem;
  onRaw = handlers.onRaw;
  onConnected = handlers.onConnected;
  onDisconnected = handlers.onDisconnected;
}

export function connect(): void {
  // Clean up existing websocket
  if (ws !== null) {
    try {
      ws.onopen = null;
      ws.onclose = null;
      ws.onerror = null;
      ws.onmessage = null;
      if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
        ws.close(1000, 'Reconnecting');
      }
    } catch (_) {
      // ignore
    }
    ws = null;
  }

  // Fix cross-domain issues
  const url = new URL(APP.chat_ws_url);
  url.hostname = window.location.hostname;
  url.port = window.location.port;
  url.protocol = window.location.protocol === 'http:' ? 'ws:' : 'wss:';

  ws = new WebSocket(url.href);

  ws.addEventListener('close', () => {
    onDisconnected?.();
    onRaw?.('Connection lost. Please wait - attempting reestablish');
    setTimeout(connect, 3000);
  });

  ws.addEventListener('error', (event) => {
    console.log('WebSocket error:', event);
  });

  ws.addEventListener('message', (event: MessageEvent) => {
    handleIncoming(event.data as string);
  });

  ws.addEventListener('open', () => {
    onConnected?.();
  });
}

export function send(message: string): void {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(message);
  }
}

export function sendChat(text: string): PendingMessage {
  const pending: PendingMessage = {
    localId: `pending-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    text,
    timestamp: Math.floor(Date.now() / 1000),
  };
  pendingMessages.push(pending);

  send(text);

  // Set echo timeout
  setTimeout(() => {
    const idx = pendingMessages.indexOf(pending);
    if (idx !== -1) {
      // Still pending — mark as potentially failed
      if (pending.element) {
        pending.element.classList.add('chat-message--retryable');
      }
    }
  }, ECHO_TIMEOUT_MS);

  return pending;
}

export function joinRoom(roomId: number): void {
  currentRoom = roomId;
  send(`/join ${roomId}`);
}

export function setCurrentRoom(roomId: number | null): void {
  currentRoom = roomId;
}

export function getCurrentRoom(): number | null {
  return currentRoom;
}

export function getPendingMessages(): PendingMessage[] {
  return pendingMessages;
}

/**
 * Try to match an incoming server message against a pending message.
 * Match by content similarity and time proximity.
 */
export function resolvePending(serverText: string, serverDate: number): PendingMessage | undefined {
  for (let i = 0; i < pendingMessages.length; i++) {
    const p = pendingMessages[i];
    if (p.text === serverText && Math.abs(serverDate - p.timestamp) < 10) {
      pendingMessages.splice(i, 1);
      return p;
    }
  }
  return undefined;
}

export function resendPendingMessages(): void {
  // On reconnect, re-send any messages that were pending
  for (const p of pendingMessages) {
    send(p.text);
  }
}

export function cleanup(): void {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.onopen = null;
    ws.onclose = null;
    ws.onerror = null;
    ws.onmessage = null;
    ws.close(1000, 'Bye!');
  }
  destroyScroll();
}

function destroyScroll(): void {
  // Placeholder — actual scroll cleanup is in scroll.ts
}

function handleIncoming(data: string): void {
  let json: ServerPayload;

  try {
    json = JSON.parse(data) as ServerPayload;
  } catch {
    // Not valid JSON — treat as plain system text
    onRaw?.(data);
    return;
  }

  // System message (from asset watcher, etc.)
  if (json.system) {
    onSystem?.(json.system);
  }

  // Standard message payload
  onMessage?.(json);
}
