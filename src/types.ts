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

export interface ChannelData {
  id: string;
  type: string;
  name: string;
  members: string[];
  created_at?: string;
  updated_at?: string;
  blocked?: boolean;
}

export interface ClientConfig {
  api_key: string;
  user_token: string;
  channels: ChannelData[];
}

export interface AuthResponse {
  userId: string;
  token: string;
}

export interface AuthRequest {
  username: string;
}

export interface CreateChannelRequest {
  channelId: string;
  channelName: string;
  userId: string;
}

export interface SendMessageRequest {
  channelId: string;
  message: string;
  userId: string;
}
