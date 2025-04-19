import React, { useEffect, useRef, useState, useTransition } from "react";
import { StreamChat, Channel, Event } from "stream-chat";
import { useVirtualizer } from "@tanstack/react-virtual";
import { motion } from "framer-motion";
import { Button } from "./components/ui/button";
import { Textarea } from "./components/ui/textarea";

// Utility to get Stream token via Tauri invoke
async function fetchToken(userId: string): Promise<string> {
  // @ts-ignore – exposed by Tauri
  return await window.__TAURI__.invoke<string>("stream_token", { userId });
}

interface ChatUIProps {
  userId: string;
  peerId: string;
  apiKey: string;
}

export default function ChatUI({ userId, peerId, apiKey }: ChatUIProps) {
  const [client, setClient] = useState<StreamChat>();
  const [channel, setChannel] = useState<Channel>();
  const [msgInput, setMsgInput] = useState("");
  const [isPending, startTransition] = useTransition();
  const messagesRef = useRef<Event[]>([]);
  const [, forceRerender] = useState(0); // local rerender trigger

  // connect Stream client
  useEffect(() => {
    (async () => {
      const token = await fetchToken(userId);
      const chat = StreamChat.getInstance(apiKey, { timeout: 3000 });
      await chat.connectUser({ id: userId }, token);

      const cid = [userId, peerId].sort().join("_");
      const chan = chat.channel("messaging", cid, {
        members: [userId, peerId],
      });
      await chan.watch();

      // prime history
      messagesRef.current = chan.state.messages;
      forceRerender((n) => n + 1);

      // listen to new messages
      const subscription = chan.on("message.new", (e) => {
        messagesRef.current.push(e);
        // schedule low‑prio render
        startTransition(() => forceRerender((n) => n + 1));
      });

      setClient(chat);
      setChannel(chan);

      return () => {
        subscription.unsubscribe();
        chat.disconnectUser();
      };
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [userId, peerId, apiKey]);

  // Virtualization with Tanstack Virtual
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: messagesRef.current.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 56,
    overscan: 8,
  });

  const sendMsg = async () => {
    if (!msgInput.trim() || !channel) return;
    const text = msgInput;
    setMsgInput("");
    // optimistic append
    messagesRef.current.push({
      type: "message.new",
      message: {
        id: `tmp-${Date.now()}`,
        text,
        user: { id: userId },
        created_at: new Date().toISOString(),
      },
    } as unknown as Event);
    forceRerender((n) => n + 1);

    await channel.sendMessage({ text });
  };

  return (
    <div className="flex flex-col h-full p-2 gap-2">
      {/* Message list */}
      <div
        ref={parentRef}
        className="flex-1 overflow-y-auto bg-neutral-900 rounded-2xl p-4"
      >
        <div
          style={{ height: `${virtualizer.getTotalSize()}px` }}
          className="relative w-full"
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const e = messagesRef.current[virtualRow.index];
            const m = (e as any)?.message;
            if (!m) return null;
            const isMe = m.user?.id === userId;
            return (
              <motion.div
                key={m.id}
                initial={{ opacity: 0, translateY: 10 }}
                animate={{ opacity: 1, translateY: 0 }}
                transition={{ duration: 0.15 }}
                className="absolute w-full"
                style={{ transform: `translateY(${virtualRow.start}px)` }}
              >
                <div
                  className={
                    "max-w-lg px-4 py-2 rounded-2xl shadow-xl text-sm " +
                    (isMe
                      ? "bg-blue-600 text-white ml-auto"
                      : "bg-neutral-700 text-gray-50 mr-auto")
                  }
                >
                  {m.text}
                </div>
              </motion.div>
            );
          })}
        </div>
      </div>

      {/* Input */}
      <div className="flex gap-2 items-end">
        <Textarea
          className="flex-1 resize-none bg-neutral-800 rounded-2xl text-gray-50"
          rows={2}
          value={msgInput}
          onChange={(e) => setMsgInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              sendMsg();
            }
          }}
        />
        <Button onClick={sendMsg} disabled={isPending || !msgInput.trim()}>
          Send
        </Button>
      </div>
    </div>
  );
}
