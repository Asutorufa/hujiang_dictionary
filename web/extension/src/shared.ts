import type { CustomLLMProvider, PromptMode } from "@/lib/translation";
import { DEFAULT_METHOD, DEFAULT_SEARCH_ENGINE } from "@/lib/translation";

export type {
  CustomLLMProvider,
  PromptMode,
  TranslationResponse,
} from "@/lib/translation";

export type ExtensionSettings = {
  baseUrl: string;
  username: string;
  token: string;
  encryptedPassword?: EncryptedSecret;
  method: string;
  srcLang: string;
  dstLang: string;
  instruction: string;
  promptMode: PromptMode;
  googleSearch: boolean;
  searchEngine: string;
  stream: boolean;
  defaultWordType: 0 | 1;
  customLLMs: CustomLLMProvider[];
};

export type EncryptedSecret = {
  iv: string;
  data: string;
  keySource?: "idb" | "storage";
};

export type TranslationPayload = {
  text: string;
  title?: string;
  url?: string;
  method?: string;
  dstLang?: string;
  promptMode?: PromptMode;
};

export type SaveWordPayload = {
  word: string;
  explain: string;
  example: string;
  wordType: 0 | 1;
};

export type TranslationHistoryRecord = {
  id: string;
  text: string;
  result: string;
  reasoning?: string;
  method: string;
  wordType: 0 | 1;
  title: string;
  url: string;
  createdAt: number;
};

export type RuntimeRequest =
  | { type: "getSettings" }
  | { type: "getHistory" }
  | { type: "clearHistory" }
  | {
      type: "saveSettings";
      settings: Partial<ExtensionSettings>;
      password?: string;
    }
  | {
      type: "login";
      baseUrl: string;
      username: string;
      password: string;
    }
  | { type: "forgetPassword" }
  | { type: "loadCustomLLMs" }
  | { type: "saveWord"; payload: SaveWordPayload }
  | { type: "openOptions" };

export type RuntimeResponse<T> =
  { ok: true; data: T } | { ok: false; error: string; authExpired?: boolean };

export type QueryPortRequest = {
  type: "translateSelection";
  payload: TranslationPayload;
};

export type QueryPortResponse =
  | { type: "start"; wordType: 0 | 1 }
  | { type: "chunk"; result: string; reasoning?: string }
  | { type: "complete"; result: string; reasoning?: string }
  | { type: "error"; error: string; authExpired?: boolean };

export const STORAGE_KEY = "dictdeck_extension_settings";
export const HISTORY_KEY = "dictdeck_extension_history";

export const DEFAULT_SETTINGS: ExtensionSettings = {
  baseUrl: "",
  username: "",
  token: "",
  method: DEFAULT_METHOD,
  srcLang: "",
  dstLang: "",
  instruction: "",
  promptMode: "default",
  googleSearch: false,
  searchEngine: DEFAULT_SEARCH_ENGINE,
  stream: true,
  defaultWordType: 0,
  customLLMs: [],
};

export function normalizeBaseUrl(baseUrl: string) {
  return baseUrl.trim().replace(/\/+$/, "");
}

export class AuthExpiredError extends Error {
  constructor(message = "Authentication expired. Please log in again.") {
    super(message);
    this.name = "AuthExpiredError";
  }
}

/** Flatten Markdown into a single readable line for compact previews. */
export function stripMarkdown(markdown: string) {
  return markdown
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, "")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/^\s{0,3}(#{1,6}|>|[-*+]|\d+\.)\s+/gm, "")
    .replace(/(\*\*|__|\*|_|~~|`)(.+?)\1/g, "$2")
    .replace(/\s+/g, " ")
    .trim();
}

const RELATIVE_UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["day", 86_400_000],
  ["hour", 3_600_000],
  ["minute", 60_000],
];

export function formatRelativeTime(timestamp: number, now = Date.now()) {
  const diff = timestamp - now;
  if (Math.abs(diff) < 60_000) return "just now";
  const formatter = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
  for (const [unit, ms] of RELATIVE_UNITS) {
    if (Math.abs(diff) >= ms || unit === "minute") {
      if (unit === "day" && Math.abs(diff) > 6 * ms) {
        return new Intl.DateTimeFormat(undefined, {
          month: "short",
          day: "numeric",
        }).format(timestamp);
      }
      return formatter.format(Math.round(diff / ms), unit);
    }
  }
  return "";
}
