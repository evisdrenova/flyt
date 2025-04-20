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
import { AuthResponse, ClientConfig } from "./types";
import type { ChannelData, Channel as StreamChannel } from "stream-chat";
import { Input } from "./components/ui/input";
import { Button } from "./components/ui/button";

export default function App() {
  const [chatClient, setChatClient] = useState<StreamChat | null>(null);
  const [currentChannel, setCurrentChannel] = useState<StreamChannel | null>(
    null
  );
  const [userId, setUserId] = useState<string>("");
  const [userToken, setUserToken] = useState<string>("");
  const [username, setUsername] = useState<string>("");
  const [channels, setChannels] = useState<ChannelData[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>("");

  // Handle login
  async function handleLogin(e: React.FormEvent) {
    e.preventDefault();

    console.log("logging user in");

    if (!username.trim()) return;

    try {
      setIsLoading(true);

      // Call to Tauri backend to authenticate user, get token, and client config
      const { userId, clientConfig } = await invoke<{
        userId: string;
        clientConfig: ClientConfig;
      }>("login_and_initialize", {
        request: {
          username: username,
        },
      });

      console.log("user authenticated, initializing chat");

      // Initialize chat client with the config from backend
      const client = StreamChat.getInstance(clientConfig.apiKey);

      // Connect user with the token
      await client.connectUser(
        {
          id: userId,
          name: username,
        },
        clientConfig.userToken
      );

      setChatClient(client);
      setUserId(userId);

      // Get channels for the user
      const userChannels = clientConfig.channels;
      setChannels(userChannels);

      // Join the first channel or general if available
      if (userChannels.length > 0) {
        const defaultChannel = userChannels[0];
        const channel = client.channel(defaultChannel.type, defaultChannel.id, {
          name: defaultChannel.name,
          members: defaultChannel.members,
        });

        await channel.watch();
        setCurrentChannel(channel);
      }

      setIsLoading(false);
    } catch (err) {
      console.error("Login failed:", err);
      setError("Authentication failed. Please try again.");
      setIsLoading(false);
    }
  }

  async function handleCreateChannel(channelName: string) {
    if (!channelName.trim() || !chatClient || !userId) return;

    const channelId = channelName.toLowerCase().replace(/\s+/g, "-");

    try {
      // Create channel via backend
      const newChannelData = await invoke<ChannelData>("create_channel", {
        request: {
          channelId,
          channelName,
          userId,
        },
      });

      // Connect to the new channel via Stream Chat SDK
      const newChannel = chatClient.channel(
        newChannelData.type,
        newChannelData.id,
        {
          name: newChannelData.name,
          members: newChannelData.members,
        }
      );

      await newChannel.watch();
      setCurrentChannel(newChannel);

      // Add to local channels list
      setChannels([...channels, newChannelData]);
    } catch (err) {
      console.error("Failed to create channel:", err);
      // Show error to user
    }
  }

  // Handle cleanup on component unmount
  React.useEffect(() => {
    return () => {
      if (chatClient) {
        chatClient.disconnectUser().then(() => {
          console.log("Disconnected from Stream Chat");
        });
      }
    };
  }, [chatClient]);

  // Render login form if not authenticated
  if (!chatClient) {
    return (
      <div className="w-full bg-zinc-800 h-screen justify-center items-center flex flex-col">
        <div className="text-gray-200">QuickChat</div>
        {error && <div className="error">{error}</div>}
        <form onSubmit={handleLogin} className="gap-2 flex flex-row">
          <Input
            type="text"
            placeholder="Username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            disabled={isLoading}
          />
          <Button type="submit" disabled={isLoading}>
            {isLoading ? "Connecting..." : "Login"}
          </Button>
        </form>
      </div>
    );
  }

  return (
    <div className="w-full h-screen">
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
