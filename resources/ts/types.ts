/** Data injected by the server template into window.APP */
export interface AppConfig {
  chat_ws_url: string;
  user: SessionUser;
}

export interface SessionUser {
  id: number;
  username: string;
  avatar_url: string;
  ignored_users: number[];
  is_staff: boolean;
}

export interface Author {
  id: number;
  username: string;
  avatar_url: string;
}

export interface SanitaryPost {
  author: Author;
  message: string;
  message_id: number;
  message_date: number;
  message_edit_date: number;
  message_raw: string;
  room_id: number;
}

export interface UserActivity {
  id: number;
  username: string;
  avatar_url: string;
  last_activity: number;
}

export interface ServerPayload {
  messages?: SanitaryPost[];
  delete?: number[];
  users?: Record<string, UserActivity | false>;
  user?: Record<string, false>;
  system?: string;
  update_id?: { old: number; new: number; room_id: number };
  can_send?: boolean;
}

export interface PendingMessage {
  localId: string;
  text: string;
  timestamp: number;
  element?: HTMLElement;
}

declare global {
  const APP: AppConfig;
  interface Window {
    MicroModal: {
      show: (id: string, config?: Record<string, unknown>) => void;
      close: (id: string) => void;
    };
  }
}
