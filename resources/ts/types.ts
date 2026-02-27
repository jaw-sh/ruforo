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
  message_uuid: string;
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

export interface RoomPermissions {
  can_view: boolean;
  can_send: boolean;
  can_edit_own: boolean;
  can_edit_other: boolean;
  can_delete_own: boolean;
  can_delete_other: boolean;
  can_report: boolean;
  can_view_deleted: boolean;
  can_undelete: boolean;
  can_motd: boolean;
}

export interface ServerPayload {
  messages?: SanitaryPost[];
  delete?: string[];
  users?: Record<string, UserActivity | false>;
  user?: Record<string, false>;
  system?: string;
  update_id?: { uuid: string; message_id: number; room_id: number };
  permissions?: RoomPermissions;
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
