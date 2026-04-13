const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL || "/api/v1";

let authToken: string | null = localStorage.getItem("cc_switch_token");

export function setAuthToken(token: string) {
  authToken = token;
  localStorage.setItem("cc_switch_token", token);
}

export function getAuthToken(): string | null {
  return authToken;
}

export function clearAuthToken() {
  authToken = null;
  localStorage.removeItem("cc_switch_token");
}

async function fetchWithAuth(
  url: string,
  options: RequestInit = {},
): Promise<Response> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string>),
  };

  if (authToken) {
    headers["Authorization"] = `Bearer ${authToken}`;
  }

  const response = await fetch(`${API_BASE_URL}${url}`, {
    ...options,
    headers,
  });

  if (response.status === 401) {
    clearAuthToken();
    // Only redirect if not already on login page to prevent redirect loops
    if (!window.location.pathname.includes('/login')) {
      window.location.href = "/login";
    }
    throw new Error("Unauthorized");
  }

  return response;
}

interface ApiEnvelope<T> {
  success: boolean;
  data: T;
  error?: string | null;
}

async function parseApiEnvelope<T>(response: Response): Promise<ApiEnvelope<T>> {
  const responseText = await response.text();
  const statusLabel = `${response.status}${response.statusText ? ` ${response.statusText}` : ""}`;

  if (!responseText) {
    throw new Error(`HTTP ${statusLabel}`);
  }

  let payload: ApiEnvelope<T>;
  try {
    payload = JSON.parse(responseText) as ApiEnvelope<T>;
  } catch {
    throw new Error(`HTTP ${statusLabel}`);
  }

  if (!payload.success) {
    throw new Error(payload.error || `HTTP ${statusLabel}`);
  }

  return payload;
}

export async function get<T>(url: string): Promise<T> {
  const response = await fetchWithAuth(url);
  const data = await parseApiEnvelope<T>(response);
  return data.data;
}

export async function post<T>(url: string, body?: unknown): Promise<T> {
  const response = await fetchWithAuth(url, {
    method: "POST",
    body: body ? JSON.stringify(body) : undefined,
  });
  const data = await parseApiEnvelope<T>(response);
  return data.data;
}

export async function put<T>(url: string, body?: unknown): Promise<T> {
  const response = await fetchWithAuth(url, {
    method: "PUT",
    body: body ? JSON.stringify(body) : undefined,
  });
  const data = await parseApiEnvelope<T>(response);
  return data.data;
}

export async function del<T>(url: string): Promise<T> {
  const response = await fetchWithAuth(url, {
    method: "DELETE",
  });
  const data = await parseApiEnvelope<T>(response);
  return data.data;
}

export function connectWebSocket(
  onMessage: (data: unknown) => void,
): () => void {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws`;
  const ws = new WebSocket(wsUrl);

  ws.onopen = () => {
    console.log("WebSocket connected");
  };

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      onMessage(data);
    } catch (e) {
      console.error("Failed to parse WebSocket message:", e);
    }
  };

  ws.onclose = () => {
    console.log("WebSocket disconnected");
  };

  ws.onerror = (error) => {
    console.error("WebSocket error:", error);
  };

  return () => {
    ws.close();
  };
}



export function connectTerminalWebSocket(
  providerId: string,
  app: string,
  onData: (data: Uint8Array) => void,
  onReady: () => void,
  onError: (error: string) => void,
  onClose: () => void,
): {
  send: (data: Uint8Array) => void;
  resize: (cols: number, rows: number) => void;
  close: () => void;
} {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const token = getAuthToken();
  const wsUrl = `${protocol}//${window.location.host}/ws/terminal?provider=${encodeURIComponent(providerId)}&app=${encodeURIComponent(app)}&token=${encodeURIComponent(token || '')}`;
  const ws = new WebSocket(wsUrl);

  ws.binaryType = 'arraybuffer';

  let isReady = false;

  ws.onopen = () => {
    console.log('Terminal WebSocket connected');
    // Send auth token as first message if needed
    if (authToken) {
      // Note: auth is handled via headers in query params for WebSocket upgrade
    }
  };

  ws.onmessage = (event) => {
    if (typeof event.data === 'string') {
      try {
        const data = JSON.parse(event.data);
        if (data.status === 'ready') {
          isReady = true;
          onReady();
        } else if (data.error) {
          onError(data.error);
        }
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e);
      }
    } else if (event.data instanceof ArrayBuffer) {
      const bytes = new Uint8Array(event.data);
      if (bytes.length > 0 && bytes[0] === 0x00) {
        // Binary protocol: 0x00 prefix for stdout/stderr data
        onData(bytes.slice(1));
      }
    }
  };

  ws.onclose = () => {
    console.log('Terminal WebSocket disconnected');
    onClose();
  };

  ws.onerror = (error) => {
    console.error('Terminal WebSocket error:', error);
    onError('Connection error');
  };

  return {
    send: (data: Uint8Array) => {
      if (ws.readyState === WebSocket.OPEN && isReady) {
        // Binary protocol: 0x00 prefix for stdin data
        const message = new Uint8Array(data.length + 1);
        message[0] = 0x00;
        message.set(data, 1);
        ws.send(message);
      }
    },
    resize: (cols: number, rows: number) => {
      if (ws.readyState === WebSocket.OPEN && isReady) {
        // Binary protocol: 0x01 prefix for resize event
        const resizeData = JSON.stringify({ cols, rows });
        const encoder = new TextEncoder();
        const jsonBytes = encoder.encode(resizeData);
        const message = new Uint8Array(jsonBytes.length + 1);
        message[0] = 0x01;
        message.set(jsonBytes, 1);
        ws.send(message);
      }
    },
    close: () => {
      ws.close();
    },
  };
}
