import { onMounted, onUnmounted } from "vue";
import { useProjectStore } from "@/stores/project";
import { useSettingsStore } from "@/stores/settings";
import { invoke } from "@tauri-apps/api/core";

export type WsStatus = "disconnected" | "connecting" | "connected" | "error";

interface SyncMessage {
  type: "project_update" | "project_snapshot" | "client_join" | "client_leave";
  version?: number;
  source?: string;
  client_id?: string;
  snapshot?: unknown;
}

export function useWebSocketSync() {
  const project = useProjectStore();
  const settings = useSettingsStore();

  let ws: WebSocket | null = null;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let recoveryTimer: ReturnType<typeof setTimeout> | null = null;
  const clientId = `gui-${Date.now()}`;
  let retryCount = 0;
  const MAX_RETRIES = 5;

  function resetConnection() {
    retryCount = 0;
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    if (ws) {
      ws.onclose = null;
      ws.close();
      ws = null;
    }
    connect();
  }

  async function connect() {
    if (retryCount >= MAX_RETRIES) {
      settings.setWsStatus("error");
      recoveryTimer = setTimeout(() => {
        retryCount = 0;
        connect();
      }, 30000);
      return;
    }
    retryCount++;
    settings.setWsStatus("connecting");

    try {
      const info = await invoke<{ port: number; token: string }>("get_ws_info");
      ws = new WebSocket(`ws://127.0.0.1:${info.port}?token=${info.token}`);

      ws.onopen = () => {
        retryCount = 0;
        settings.setWsStatus("connected");
        ws?.send(
          JSON.stringify({
            type: "client_join",
            client_id: clientId,
          })
        );
      };

      ws.onmessage = (event) => {
        try {
          const msg: SyncMessage = JSON.parse(event.data);
          handleServerMessage(msg);
        } catch {
          console.error("[WS] Failed to parse message");
        }
      };

      ws.onerror = () => {
        settings.setWsStatus("error");
      };

      ws.onclose = () => {
        settings.setWsStatus("disconnected");
        reconnectTimer = setTimeout(connect, 3000);
      };
    } catch {
      settings.setWsStatus("error");
      reconnectTimer = setTimeout(connect, 3000);
    }
  }

  function handleServerMessage(msg: SyncMessage) {
    switch (msg.type) {
      case "project_snapshot":
      case "project_update": {
        // Accept all updates — backend handles conflict resolution via Last-Writer-Wins
        project.refreshElements();
        project.debouncedFetchPreview();
        break;
      }
      case "client_join":
      case "client_leave":
        break;
    }
  }

  function sendUpdate() {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(
        JSON.stringify({
          type: "project_update",
          source: clientId,
          snapshot: project.elements,
        })
      );
    }
  }

  onMounted(() => {
    connect();
  });

  onUnmounted(() => {
    if (reconnectTimer) clearTimeout(reconnectTimer);
    if (recoveryTimer) clearTimeout(recoveryTimer);
    if (ws) {
      ws.onclose = null;
      ws.close();
    }
  });

  return { sendUpdate, resetConnection };
}
