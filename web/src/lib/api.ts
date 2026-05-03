import { TOKEN_KEY } from "./constants";
import { consumeSseStream } from "./translation";

export async function authorizedRequest(
  input: RequestInfo | URL,
  init?: RequestInit,
): Promise<Response> {
  const token = localStorage.getItem(TOKEN_KEY);

  const headers = new Headers(init?.headers);

  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  const newInit: RequestInit = {
    ...init,
    headers,
  };

  const response = await fetch(input, newInit);

  if (response.status === 401) {
    localStorage.removeItem(TOKEN_KEY);
    window.dispatchEvent(new CustomEvent("unauthorized"));
  }

  return response;
}
export async function streamRequest<T>(
  response: Response,
  onData: (data: T) => void,
) {
  await consumeSseStream(response, onData);
}
