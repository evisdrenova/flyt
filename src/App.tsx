// import { useState } from "react";
// import { invoke } from "@tauri-apps/api/core";
import "./globals.css";
import ChatUI from "./ChatUi";

function App() {
  return (
    <main className="container">
      <div>Chat UI</div>
      <ChatUI userId="" peerId="" apiKey="" />
    </main>
  );
}

export default App;
