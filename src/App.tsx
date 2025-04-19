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
import "stream-chat-react/dist/css/index.css";
import "./App.css";
import { AuthResponse } from "./types";
import type { Channel as StreamChannel } from "stream-chat";

export default function App() {
  const [chatClient, setChatClient] = useState<StreamChat | null>(null);
  const [currentChannel, setCurrentChannel] = useState<StreamChannel | null>(
    null
  );
  const [userId, setUserId] = useState<string>("");
  const [userToken, setUserToken] = useState<string>("");
  const [username, setUsername] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string>("");
  const [apiKey, setApiKey] = useState<string>("");

  // Fetch API key from backend
  useEffect(() => {
    async function fetchApiKey() {
      try {
        const key = await invoke<string>("get_stream_api_key");
        setApiKey(key);
      } catch (err) {
        console.error("Failed to fetch API key:", err);
        setError("Failed to connect to application server");
      }
    }

    fetchApiKey();
  }, []);

  // Initialize Stream Chat client when user credentials are available
  useEffect(() => {
    async function initializeChat() {
      if (!userId || !userToken || !apiKey) return;

      try {
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
        const channel = client.channel("team", "vista", {
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
  }, [userId, userToken, username, apiKey]);

  // Handle login
  async function handleLogin(e: React.FormEvent) {
    e.preventDefault();

    if (!username.trim()) return;

    try {
      setIsLoading(true);

      // Call to Tauri backend to authenticate user and get token
      const { userId, token } = await invoke<AuthResponse>(
        "authenticate_user",
        {
          username: username,
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
      <div className="login-container">
        <h1>QuickChat</h1>
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

  // Render chat interface if authenticated
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
