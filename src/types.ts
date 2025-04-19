// src/types.ts
import { StreamChat, UserResponse } from "stream-chat";

export interface User {
  id: string;
  name: string;
  image?: string;
  status?: "online" | "offline" | "away";
}

export interface Message {
  id: string;
  text: string;
  user: User;
  created_at: string;
  updated_at?: string;
  attachments?: Attachment[];
  mentioned_users?: User[];
  thread_count?: number;
  reaction_counts?: Record<string, number>;
}

export interface Attachment {
  type: "image" | "file" | "link";
  title?: string;
  description?: string;
  url: string;
  name?: string;
  size?: number;
  mime_type?: string;
}

export interface Channel {
  id: string;
  type: "team" | "messaging";
  name: string;
  image?: string;
  description?: string;
  member_count: number;
  created_by: User;
  created_at: string;
  updated_at: string;
  last_message_at?: string;
  unread_count?: number;
}

export interface AuthResponse {
  userId: string;
  token: string;
}

export interface InitChatResponse {
  apiKey: string;
  channelId: string;
}

export interface CreateChannelRequest {
  channelId: string;
  channelName: string;
  members: string[];
}

export interface SendMessageRequest {
  channelId: string;
  message: string;
}
