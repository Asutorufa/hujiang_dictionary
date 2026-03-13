import { TOKEN_KEY } from "./constants";

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
  const reader = response.body?.getReader();
  if (!reader) {
    throw new Error("Failed to get stream reader");
  }

  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const line of lines) {
      if (line.startsWith("data: ")) {
        try {
          const data = JSON.parse(line.slice(6)) as T;
          onData(data);
        } catch (e) {
          console.error("Error parsing stream data:", e);
        }
      }
    }
  }
}
