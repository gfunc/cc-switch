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

export async function get<T>(url: string): Promise<T> {
  const response = await fetchWithAuth(url);
  const data = await response.json();
  if (!data.success) {
    throw new Error(data.error || "Request failed");
  }
  return data.data;
}

export async function post<T>(url: string, body?: unknown): Promise<T> {
  const response = await fetchWithAuth(url, {
    method: "POST",
    body: body ? JSON.stringify(body) : undefined,
  });
  const data = await response.json();
  if (!data.success) {
    throw new Error(data.error || "Request failed");
  }
  return data.data;
}

export async function put<T>(url: string, body?: unknown): Promise<T> {
  const response = await fetchWithAuth(url, {
    method: "PUT",
    body: body ? JSON.stringify(body) : undefined,
  });
  const data = await response.json();
  if (!data.success) {
    throw new Error(data.error || "Request failed");
  }
  return data.data;
}

export async function del<T>(url: string): Promise<T> {
  const response = await fetchWithAuth(url, {
    method: "DELETE",
  });
  const data = await response.json();
  if (!data.success) {
    throw new Error(data.error || "Request failed");
  }
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
