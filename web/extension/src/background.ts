import {
  HISTORY_LIMIT,
  buildTranslationQueryBody,
  consumeSseStream,
  parseCustomMethod,
  type TranslationResponse,
} from "@/lib/translation";
import {
  AuthExpiredError,
  DEFAULT_SETTINGS,
  HISTORY_KEY,
  STORAGE_KEY,
  type CustomLLMProvider,
  type EncryptedSecret,
  type ExtensionSettings,
  type QueryPortRequest,
  type QueryPortResponse,
  type RuntimeRequest,
  type RuntimeResponse,
  type TranslationHistoryRecord,
  type TranslationPayload,
  normalizeBaseUrl,
} from "./shared";
import { getExtensionApi, type ExtensionPort } from "./webextension";

const api = getExtensionApi();

type StoredSettings = Record<typeof STORAGE_KEY, Partial<ExtensionSettings>>;
type StoredHistory = Record<typeof HISTORY_KEY, TranslationHistoryRecord[]>;
type StoredPasswordKey = Record<
  typeof PASSWORD_RAW_KEY_STORAGE_KEY,
  string | undefined
>;
type LoginCredentials = {
  baseUrl: string;
  username: string;
  password: string;
};

const PASSWORD_RAW_KEY_STORAGE_KEY = "dictdeck_extension_password_raw_key";
const CRYPTO_TIMEOUT_MS = 2500;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  message: string,
) {
  let timeoutId: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<T>((_, reject) => {
    timeoutId = setTimeout(() => reject(new Error(message)), timeoutMs);
  });

  return Promise.race([promise, timeout]).finally(() => {
    if (timeoutId !== undefined) {
      clearTimeout(timeoutId);
    }
  });
}

async function getStoragePasswordKey() {
  const stored = (await api.storage.local.get(
    PASSWORD_RAW_KEY_STORAGE_KEY,
  )) as StoredPasswordKey;
  const existingKey = stored?.[PASSWORD_RAW_KEY_STORAGE_KEY];
  if (existingKey) {
    return crypto.subtle.importKey(
      "raw",
      base64ToBytes(existingKey),
      { name: "AES-GCM" },
      false,
      ["encrypt", "decrypt"],
    );
  }

  const key = await crypto.subtle.generateKey(
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"],
  );
  const raw = await crypto.subtle.exportKey("raw", key);
  await api.storage.local.set({
    [PASSWORD_RAW_KEY_STORAGE_KEY]: bytesToBase64(new Uint8Array(raw)),
  });
  return key;
}

async function getPasswordKey(source?: EncryptedSecret["keySource"]) {
  if (source === "idb") {
    throw new Error(
      "Legacy encrypted password storage is no longer supported in Safari. Please enter the password again and save.",
    );
  }

  return {
    key: await withTimeout(
      getStoragePasswordKey(),
      CRYPTO_TIMEOUT_MS,
      "Encrypted password key storage timed out.",
    ),
    source: "storage" as const,
  };
}

function bytesToBase64(bytes: Uint8Array) {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

function base64ToBytes(value: string) {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

async function encryptPassword(password: string): Promise<EncryptedSecret> {
  const { key, source } = await getPasswordKey();
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(password);
  const encrypted = await withTimeout(
    crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, encoded),
    CRYPTO_TIMEOUT_MS,
    "Password encryption timed out.",
  );
  return {
    iv: bytesToBase64(iv),
    data: bytesToBase64(new Uint8Array(encrypted)),
    keySource: source,
  };
}

async function decryptPassword(secret: EncryptedSecret) {
  const { key } = await getPasswordKey(secret.keySource);
  const decrypted = await withTimeout(
    crypto.subtle.decrypt(
      { name: "AES-GCM", iv: base64ToBytes(secret.iv) },
      key,
      base64ToBytes(secret.data),
    ),
    CRYPTO_TIMEOUT_MS,
    "Password decryption timed out.",
  );
  return new TextDecoder().decode(decrypted);
}

async function readSettings(): Promise<ExtensionSettings> {
  const stored = (await api.storage.local.get(STORAGE_KEY)) as StoredSettings;
  return {
    ...DEFAULT_SETTINGS,
    ...(stored?.[STORAGE_KEY] ?? {}),
  };
}

async function writeSettings(settings: Partial<ExtensionSettings>) {
  const current = await readSettings();
  const next: ExtensionSettings = {
    ...current,
    ...settings,
    baseUrl:
      settings.baseUrl !== undefined
        ? normalizeBaseUrl(settings.baseUrl)
        : current.baseUrl,
    defaultWordType:
      settings.defaultWordType !== undefined
        ? settings.defaultWordType === 1
          ? 1
          : 0
        : current.defaultWordType,
  };
  if (
    Object.prototype.hasOwnProperty.call(settings, "encryptedPassword") &&
    settings.encryptedPassword === undefined
  ) {
    delete next.encryptedPassword;
  }

  await api.storage.local.set({ [STORAGE_KEY]: next });
  return next;
}

async function readHistory(): Promise<TranslationHistoryRecord[]> {
  const stored = (await api.storage.local.get(HISTORY_KEY)) as StoredHistory;
  return stored?.[HISTORY_KEY] ?? [];
}

async function writeHistory(history: TranslationHistoryRecord[]) {
  await api.storage.local.set({
    [HISTORY_KEY]: history.slice(0, HISTORY_LIMIT),
  });
}

function endpoint(settings: ExtensionSettings, path: string) {
  if (!settings.baseUrl) {
    throw new Error("Please configure the DictDeck service URL first.");
  }
  return `${settings.baseUrl}${path}`;
}

async function fetchLoginToken(credentials: LoginCredentials) {
  const baseUrl = normalizeBaseUrl(credentials.baseUrl);
  if (!baseUrl) {
    throw new Error("Service URL is required.");
  }

  const response = await fetch(`${baseUrl}/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      username: credentials.username,
      password: credentials.password,
    }),
  });

  if (!response.ok) {
    throw new Error(`(${response.status}) ${await response.text()}`);
  }

  const data = (await response.json()) as { token: string };
  return { baseUrl, token: data.token };
}

async function refreshToken(settings: ExtensionSettings) {
  if (!settings.encryptedPassword) {
    await writeSettings({ token: "" });
    throw new AuthExpiredError("Authentication expired. Please log in again.");
  }

  try {
    const password = await decryptPassword(settings.encryptedPassword);
    const { baseUrl, token } = await fetchLoginToken({
      baseUrl: settings.baseUrl,
      username: settings.username,
      password,
    });
    return writeSettings({ baseUrl, token });
  } catch (error) {
    await writeSettings({ token: "" });
    throw new AuthExpiredError(
      error instanceof Error
        ? `Auto login failed: ${error.message}`
        : "Auto login failed.",
    );
  }
}

async function authorizedFetch(
  settings: ExtensionSettings,
  path: string,
  body: Record<string, unknown> | undefined,
  retryAuth = true,
) {
  const headers = new Headers({ "Content-Type": "application/json" });
  if (settings.token) {
    headers.set("Authorization", `Bearer ${settings.token}`);
  }

  const response = await fetch(endpoint(settings, path), {
    method: "POST",
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  if (response.status === 401) {
    if (!retryAuth) {
      await writeSettings({ token: "" });
      throw new AuthExpiredError();
    }
    const refreshed = await refreshToken(settings);
    return authorizedFetch(refreshed, path, body, false);
  }

  return response;
}

async function request<T>(
  settings: ExtensionSettings,
  path: string,
  body: Record<string, unknown> | undefined,
): Promise<T> {
  const response = await authorizedFetch(settings, path, body);

  if (!response.ok) {
    throw new Error(`(${response.status}) ${await response.text()}`);
  }

  return (await response.json()) as T;
}

function buildQueryOptions(
  settings: ExtensionSettings,
  text: string,
  payload?: TranslationPayload,
) {
  const selectedMethod = payload?.method?.trim() || settings.method;
  return {
    method: selectedMethod,
    text,
    srcLang: settings.srcLang,
    dstLang: payload?.dstLang ?? settings.dstLang,
    instruction: settings.instruction,
    promptMode: payload?.promptMode ?? settings.promptMode,
    googleSearch: settings.googleSearch,
    searchEngine: settings.searchEngine,
  };
}

function shouldStream(settings: ExtensionSettings, method: string) {
  return settings.stream && parseCustomMethod(method) !== undefined;
}

function truncate(value: string, limit: number) {
  return value.length > limit ? `${value.slice(0, limit)}...` : value;
}

async function addHistory(
  method: string,
  wordType: 0 | 1,
  payload: TranslationPayload,
  response: TranslationResponse,
) {
  const history = await readHistory();
  const record: TranslationHistoryRecord = {
    id: `${Date.now()}-${crypto.randomUUID()}`,
    text: truncate(payload.text, 2000),
    result: truncate(response.result, 4000),
    reasoning: response.reasoning
      ? truncate(response.reasoning, 2000)
      : undefined,
    method,
    wordType,
    title: payload.title || "",
    url: payload.url || "",
    createdAt: Date.now(),
  };
  await writeHistory([record, ...history]);
}

async function translate(payload: TranslationPayload) {
  const settings = await readSettings();
  const text = payload.text.trim();
  const selectedMethod = payload.method?.trim() || settings.method;
  if (!text) {
    throw new Error("Selected text is empty.");
  }

  const response = await request<TranslationResponse>(
    settings,
    "/word/query",
    buildTranslationQueryBody(buildQueryOptions(settings, text, payload)),
  );
  await addHistory(
    selectedMethod,
    settings.defaultWordType,
    payload,
    response,
  ).catch((error: unknown) => {
    console.warn("Failed to save translation history", error);
  });
  return response;
}

async function streamTranslate(
  port: ExtensionPort,
  payload: TranslationPayload,
) {
  const settings = await readSettings();
  const text = payload.text.trim();
  const selectedMethod = payload.method?.trim() || settings.method;
  if (!text) {
    throw new Error("Selected text is empty.");
  }

  port.postMessage({
    type: "start",
    wordType: settings.defaultWordType,
  } satisfies QueryPortResponse);

  if (!shouldStream(settings, selectedMethod)) {
    const response = await translate(payload);
    port.postMessage({
      type: "complete",
      result: response.result,
      reasoning: response.reasoning,
    } satisfies QueryPortResponse);
    return;
  }

  const response = await authorizedFetch(
    settings,
    "/word/query_stream",
    buildTranslationQueryBody(buildQueryOptions(settings, text, payload)),
  );
  if (!response.ok) {
    throw new Error(`(${response.status}) ${await response.text()}`);
  }

  let result = "";
  let reasoning = "";

  await consumeSseStream<TranslationResponse>(response, (data) => {
    const nextResult = data.result || "";
    const nextReasoning = data.reasoning || "";
    result += nextResult;
    reasoning += nextReasoning;
    port.postMessage({
      type: "chunk",
      result: nextResult,
      reasoning: data.reasoning,
    } satisfies QueryPortResponse);
  });

  port.postMessage({
    type: "complete",
    result,
    reasoning,
  } satisfies QueryPortResponse);
  await addHistory(selectedMethod, settings.defaultWordType, payload, {
    result,
    reasoning,
  }).catch((error: unknown) => {
    console.warn("Failed to save translation history", error);
  });
}

async function login(message: Extract<RuntimeRequest, { type: "login" }>) {
  const current = await readSettings();
  const password =
    message.password ||
    (current.encryptedPassword
      ? await decryptPassword(current.encryptedPassword)
      : "");
  if (!password) {
    throw new Error("Password is required for first login.");
  }

  const { baseUrl, token } = await fetchLoginToken({
    baseUrl: message.baseUrl || current.baseUrl,
    username: message.username || current.username,
    password,
  });
  return writeSettings({
    baseUrl,
    username: message.username || current.username,
    token,
    encryptedPassword: message.password
      ? await encryptPassword(message.password)
      : current.encryptedPassword,
  });
}

async function saveSettings(
  message: Extract<RuntimeRequest, { type: "saveSettings" }>,
) {
  const nextSettings: Partial<ExtensionSettings> = { ...message.settings };
  if (message.password) {
    nextSettings.encryptedPassword = await encryptPassword(message.password);
  }
  return writeSettings(nextSettings);
}

async function forgetPassword() {
  return writeSettings({
    encryptedPassword: undefined,
    token: "",
  });
}

async function loadCustomLLMs() {
  const settings = await readSettings();
  const customLLMs = await request<CustomLLMProvider[]>(
    settings,
    "/word/ai_custom",
    undefined,
  );
  return writeSettings({ customLLMs });
}

async function saveWord(
  payload: Extract<RuntimeRequest, { type: "saveWord" }>["payload"],
) {
  const settings = await readSettings();
  await request<object>(settings, "/word/save", {
    word: payload.word,
    explain: payload.explain,
    example: payload.example,
    type: payload.wordType,
  });
  return {};
}

async function handleMessage(message: unknown): Promise<unknown> {
  if (!isRecord(message) || typeof message.type !== "string") {
    throw new Error("Invalid extension message.");
  }

  const requestMessage = message as RuntimeRequest;
  switch (requestMessage.type) {
    case "getSettings":
      return readSettings();
    case "getHistory":
      return readHistory();
    case "clearHistory":
      await writeHistory([]);
      return [];
    case "saveSettings":
      return saveSettings(requestMessage);
    case "login":
      return login(requestMessage);
    case "forgetPassword":
      return forgetPassword();
    case "loadCustomLLMs":
      return loadCustomLLMs();
    case "saveWord":
      return saveWord(requestMessage.payload);
    case "openOptions":
      await api.runtime.openOptionsPage();
      return {};
    default:
      throw new Error("Unsupported extension message.");
  }
}

api.runtime.onMessage?.addListener((message, _sender, sendResponse) => {
  handleMessage(message)
    .then((data) => {
      sendResponse({ ok: true, data } satisfies RuntimeResponse<unknown>);
    })
    .catch((error: unknown) => {
      sendResponse({
        ok: false,
        error: error instanceof Error ? error.message : String(error),
        authExpired: error instanceof AuthExpiredError,
      } satisfies RuntimeResponse<unknown>);
    });

  return true;
});

api.runtime.onConnect?.addListener((port) => {
  if (port.name !== "dictdeck-query") return;

  port.onMessage.addListener((message: unknown) => {
    const requestMessage = message as QueryPortRequest;
    if (requestMessage.type !== "translateSelection") return;

    streamTranslate(port, requestMessage.payload).catch((error: unknown) => {
      port.postMessage({
        type: "error",
        error: error instanceof Error ? error.message : String(error),
        authExpired: error instanceof AuthExpiredError,
      } satisfies QueryPortResponse);
    });
  });
});
