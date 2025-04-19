// import { useState } from "react";
// import { invoke } from "@tauri-apps/api/core";
import "./globals.css";
import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { StreamChat } from "stream-chat";
import {
  Chat,
  Channel,
  ChannelHeader,
  ChannelList,
  MessageInput,
  MessageList,
  Thread,
  Window,
} from "stream-chat-react";
import "./App.css";
import { AuthResponse, AuthRequest, InitChatResponse } from "./types";
import type { Channel as StreamChannel } from "stream-chat";

export default function App() {
  const [chatClient, setChatClient] = useState<StreamChat | null>(null);
  const [currentChannel, setCurrentChannel] = useState<StreamChannel | null>(
    null
  );
  const [userId, setUserId] = useState<string>("");
  const [userToken, setUserToken] = useState<string>("");
  const [username, setUsername] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>("");

  // Initialize Stream Chat client when user credentials are available
  useEffect(() => {
    async function initializeChat() {
      if (!userId || !userToken) return;

      try {
        // Get client details from backend instead of using API key directly
        const { apiKey, channelId } = await invoke<InitChatResponse>(
          "initialize_chat",
          {
            userId,
            username,
          }
        );

        const client = StreamChat.getInstance(apiKey);
        await client.connectUser(
          {
            id: userId,
            name: username,
          },
          userToken
        );

        setChatClient(client);

        // Auto-join a general channel
        const channel = client.channel("team", channelId, {
          name: "General",
          members: [userId],
        });

        await channel.watch();
        setCurrentChannel(channel);
        setIsLoading(false);
      } catch (err) {
        console.error("Error connecting to Stream Chat:", err);
        setError("Failed to connect to chat server");
        setIsLoading(false);
      }
    }

    initializeChat();

    // Cleanup on unmount
    return () => {
      if (chatClient) {
        chatClient.disconnectUser().then(() => {
          console.log("Disconnected from Stream Chat");
        });
      }
    };
  }, [userId, userToken, username]);

  console.log("user id", userId, userToken);

  // Handle login
  async function handleLogin(e: React.FormEvent) {
    e.preventDefault();

    console.log("try to login");

    if (!username.trim()) return;

    try {
      setIsLoading(true);

      const request: AuthRequest = {
        username: username,
      };

      // Call to Tauri backend to authenticate user and get token
      const { userId, token } = await invoke<AuthResponse>(
        "authenticate_user",
        {
          request: request,
        }
      );

      setUserId(userId);
      setUserToken(token);
    } catch (err) {
      console.error("Login failed:", err);
      setError("Authentication failed. Please try again.");
      setIsLoading(false);
    }
  }

  // Handle channel creation
  async function handleCreateChannel(channelName: string) {
    if (!channelName.trim() || !chatClient || !userId) return;

    const channelId = channelName.toLowerCase().replace(/\s+/g, "-");

    try {
      // Create channel via backend
      await invoke<void>("create_channel", {
        channelId,
        channelName,
        members: [userId],
        userId,
      });

      // Now connect to it via Stream Chat SDK
      const newChannel = chatClient.channel("team", channelId, {
        name: channelName,
        members: [userId],
      });

      await newChannel.watch();
      setCurrentChannel(newChannel);
    } catch (err) {
      console.error("Failed to create channel:", err);
      // Show error to user
    }
  }

  // Render login form if not authenticated
  if (!chatClient) {
    return (
      <div className="container bg-zinc-800">
        <div className="text-gray-200">QuickChat</div>
        {error && <div className="error">{error}</div>}
        <form onSubmit={handleLogin}>
          <input
            type="text"
            placeholder="Username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            disabled={isLoading}
          />
          <button type="submit" disabled={isLoading}>
            {isLoading ? "Connecting..." : "Login"}
          </button>
        </form>
      </div>
    );
  }

  return (
    <div className="chat-container">
      <Chat client={chatClient} theme="messaging dark">
        <div className="chat-wrapper">
          <div className="channel-list">
            <ChannelList
              filters={{
                type: "team",
                members: { $in: [userId] },
              }}
              sort={{ last_message_at: -1 }}
              Preview={(previewProps) => {
                const channel = previewProps.channel;
                return (
                  <div
                    className="channel-preview"
                    onClick={() => setCurrentChannel(channel)}
                  >
                    # {channel.data?.name || channel.id}
                  </div>
                );
              }}
            />
            <div className="create-channel">
              <button
                onClick={() => {
                  const name = prompt("Enter channel name:");
                  if (name) handleCreateChannel(name);
                }}
              >
                + New Channel
              </button>
            </div>
          </div>

          {currentChannel && (
            <Channel channel={currentChannel}>
              <Window>
                <ChannelHeader />
                <MessageList />
                <MessageInput focus />
              </Window>
              <Thread />
            </Channel>
          )}
        </div>
      </Chat>
    </div>
  );
}
