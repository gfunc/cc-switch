import { SignJWT } from "jose";

const BASE_URL = process.env.API_BASE_URL ?? "http://localhost:13001";
const JWT_SECRET = process.env.JWT_SECRET ?? "cc-switch-dev-secret";

/**
 * Generate a short-lived JWT signed with the same secret as the backend.
 */
export async function generateToken(subject = "test-user"): Promise<string> {
  const secret = new TextEncoder().encode(JWT_SECRET);
  return new SignJWT({ sub: subject })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt()
    .setExpirationTime("1h")
    .sign(secret);
}

/**
 * GET request with optional Bearer token.
 */
export async function apiGet(
  path: string,
  token?: string,
): Promise<Response> {
  const headers: Record<string, string> = { "Content-Type": "application/json" };
  if (token) headers["Authorization"] = `Bearer ${token}`;
  return fetch(`${BASE_URL}${path}`, { headers });
}

/**
 * POST/PUT/DELETE request with optional Bearer token.
 */
export async function apiRequest(
  method: string,
  path: string,
  body?: unknown,
  token?: string,
): Promise<Response> {
  const headers: Record<string, string> = { "Content-Type": "application/json" };
  if (token) headers["Authorization"] = `Bearer ${token}`;
  return fetch(`${BASE_URL}${path}`, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
}
