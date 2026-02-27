/**
 * Chat entry point.
 * Wires together all modules and initializes the chat application.
 */

import '../css/main.scss';

import type { ServerPayload, RoomPermissions } from './types';
import * as ws from './ws';
import * as messages from './messages';
import * as users from './users';
import * as scroll from './scroll';
import * as mentions from './mentions';
import { initToolbar } from './toolbar';
import { initInput } from './input';

let roomPerms: RoomPermissions = {
  can_view: false,
  can_send: false,
  can_edit_own: false,
  can_edit_other: false,
  can_delete_own: false,
  can_delete_other: false,
  can_report: false,
  can_view_deleted: false,
  can_undelete: false,
  can_motd: false,
};

export function getRoomPermissions(): RoomPermissions {
  return roomPerms;
}

document.addEventListener('DOMContentLoaded', () => {
  const scrollEl = document.getElementById('chat-scroller')!;

  // Initialize scroll management
  scroll.initScroll(scrollEl);

  // Initialize toolbar
  initToolbar();

  // Initialize input handling
  initInput();

  // Configure WebSocket handlers
  ws.configure({
    onMessage: handleServerPayload,
    onSystem: handleSystemMessage,
    onRaw: handleRawMessage,
    onConnected: handleConnected,
    onDisconnected: handleDisconnected,
  });

  // Room buttons via hash
  window.addEventListener('hashchange', joinByHash, false);

  // Global escape to cancel edits
  window.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      messages.messageEditReverse();
    }
  });

  // Scroll to bottom on resize
  window.addEventListener('resize', () => {
    scroll.scrollToNew();
  });

  // Clean up on page unload
  window.addEventListener('beforeunload', () => {
    ws.cleanup();
    scroll.destroyScroll();
  });

  // Connect
  messages.messagePush({ message: 'Connecting to SneedChat...' } as never, undefined);
  ws.connect();
});

function handleServerPayload(data: ServerPayload): void {
  if (data.messages) {
    for (const msg of data.messages) {
      messages.messagePush(msg, msg.author);
    }
  }

  if (data.delete) {
    for (const uuid of data.delete) {
      messages.messageDelete(uuid);
    }
  }

  if (data.users) {
    for (const [key, value] of Object.entries(data.users)) {
      users.userActivity(key, value);
    }
    users.userActivitySort();
  }

  if (data.user) {
    // Disconnect notifications: {"user": {"123": false}}
    for (const [key, value] of Object.entries(data.user)) {
      if (value === false) {
        users.userActivity(key, false);
      }
    }
  }

  if (data.update_id) {
    messages.resolveMessageId(data.update_id.uuid, data.update_id.message_id);
  }

  if (data.permissions) {
    roomPerms = data.permissions;
    setChatInputEnabled(roomPerms.can_send);
  }
}

function handleSystemMessage(text: string): void {
  messages.messagePush({ message: text } as never, undefined);
}

function handleRawMessage(text: string): void {
  messages.messagePush({ message: text } as never, undefined);
}

function handleConnected(): void {
  const room = ws.getCurrentRoom();

  if (room !== null) {
    // Reconnecting while in a room — rejoin so the server knows.
    messages.messagePush({ message: 'Reconnected.' } as never, undefined);
    ws.joinRoom(room);
  } else if (joinByHash()) {
    messages.messagePush({ message: 'Connected!' } as never, undefined);
  } else {
    messages.messagePush({ message: 'Connected! You may now join a room.' } as never, undefined);
  }
}

function handleDisconnected(): void {
  users.userActivityDelete();
  ws.clearPending();
}

function setChatInputEnabled(enabled: boolean): void {
  const form = document.querySelector('.chat-form') as HTMLElement | null;
  if (!form) return;

  const input = form.querySelector('.chat-input') as HTMLTextAreaElement | null;
  if (input) {
    input.disabled = !enabled;
    input.placeholder = enabled ? '' : 'You do not have permission to send messages in this room.';
  }
}

function joinByHash(): boolean {
  const hash = window.location.hash.substring(1);
  const roomId = parseInt(hash, 10);

  if (roomId > 0) {
    scroll.resetScrollAnchor();
    messages.messagesDelete();
    users.userActivityDelete();
    mentions.dismiss();
    roomPerms = {
      can_view: false,
      can_send: false,
      can_edit_own: false,
      can_edit_other: false,
      can_delete_own: false,
      can_delete_other: false,
      can_report: false,
      can_view_deleted: false,
      can_undelete: false,
      can_motd: false,
    };
    ws.joinRoom(roomId);
    return true;
  }

  return false;
}
