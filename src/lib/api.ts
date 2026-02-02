export async function authorizedRequest(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
  const token = localStorage.getItem("token");

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
    localStorage.removeItem("token");
    window.location.hash = "/login";
  }

  return response;
}
