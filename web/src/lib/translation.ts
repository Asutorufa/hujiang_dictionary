export type PromptMode = "default" | "translate" | "explain" | "detailed";

export type CustomLLM = {
  name: string;
  model: string;
};

export type CustomLLMProvider = {
  name: string;
  models: string[];
};

export type TranslationResponse = {
  result: string;
  reasoning?: string;
};

export type TranslationQueryOptions = {
  method: string;
  text: string;
  srcLang?: string;
  dstLang?: string;
  instruction?: string;
  promptMode?: PromptMode;
  googleSearch?: boolean;
  searchEngine?: string;
};

export const HISTORY_LIMIT = 100;

export const BUILTIN_METHODS = [
  { value: "google", label: "Google Translate" },
  { value: "googlev1", label: "Google Translate (old API)" },
  { value: "m2m100_1_2b", label: "m2m100-1.2b" },
  { value: "weblio", label: "Weblio" },
  { value: "ktbk", label: "コトバンク" },
  { value: "jc", label: "Japanese -> Chinese" },
  { value: "cj", label: "Japanese <- Chinese" },
  { value: "kr", label: "Korean <-> Chinese" },
  { value: "en", label: "English <-> Chinese" },
] as const;

export const LANGUAGES = [
  { value: "", label: "Auto" },
  { value: "ja", label: "Japanese" },
  { value: "zh", label: "Chinese" },
  { value: "zh-CN", label: "Chinese (Simplified)" },
  { value: "zh-TW", label: "Chinese (Traditional)" },
  { value: "en", label: "English" },
  { value: "ko", label: "Korean" },
  { value: "fr", label: "French" },
  { value: "de", label: "German" },
  { value: "es", label: "Spanish" },
  { value: "it", label: "Italian" },
  { value: "pt", label: "Portuguese" },
  { value: "ru", label: "Russian" },
  { value: "vi", label: "Vietnamese" },
  { value: "th", label: "Thai" },
  { value: "id", label: "Indonesian" },
] as const;

export const PROMPT_MODES: Array<{ value: PromptMode; label: string }> = [
  { value: "default", label: "Default" },
  { value: "translate", label: "Simple Translation" },
  { value: "explain", label: "Word-by-Word Explanation" },
  { value: "detailed", label: "Detailed Analysis" },
];

export const SEARCH_ENGINES = [
  { value: "duckduckgo", label: "DuckDuckGo" },
  { value: "google", label: "Google HTML" },
  { value: "google_api", label: "Google API" },
] as const;

export const DEFAULT_METHOD = "ktbk";
export const DEFAULT_SEARCH_ENGINE = SEARCH_ENGINES[0].value;

const BUILTIN_METHOD_LABELS = new Map(
  BUILTIN_METHODS.map((method) => [method.value, method.label]),
);

export function customMethodValue(llm: CustomLLM) {
  return `custom:${encodeURIComponent(llm.name)}:${encodeURIComponent(llm.model)}`;
}

export function legacyCustomMethodValue(llm: CustomLLM) {
  return `custom-${llm.name}-${llm.model}`;
}

export function parseCustomMethod(value: string): CustomLLM | undefined {
  if (!value.startsWith("custom:")) return undefined;

  const parts = value.split(":");
  if (parts.length !== 3) return undefined;

  const [, name, model] = parts;
  if (!name || !model) return undefined;

  return {
    name: decodeURIComponent(name),
    model: decodeURIComponent(model),
  };
}

export function parseLegacyCustomMethod(
  value: string,
  providers: CustomLLMProvider[],
): CustomLLM | undefined {
  if (!value.startsWith("custom-")) return undefined;

  for (const provider of providers) {
    for (const model of provider.models) {
      const candidate = { name: provider.name, model };
      if (legacyCustomMethodValue(candidate) === value) {
        return candidate;
      }
    }
  }

  return undefined;
}

export function methodLabel(value: string) {
  const customLLM = parseCustomMethod(value);
  if (customLLM) {
    return `${customLLM.model} (${customLLM.name})`;
  }

  if (value.startsWith("custom-")) {
    return "Custom LLM";
  }

  return BUILTIN_METHOD_LABELS.get(value) ?? value;
}

export function buildTranslationQueryBody(options: TranslationQueryOptions) {
  const method = options.method.trim();
  const customLLM = parseCustomMethod(method);
  const body: Record<string, unknown> = {
    method: customLLM ? "custom_llm" : method,
    word: options.text,
  };

  if (options.srcLang) body.src_lang = options.srcLang;
  if (options.dstLang) body.dst_lang = options.dstLang;
  if (options.instruction) body.instruction = options.instruction;
  if (options.promptMode && options.promptMode !== "default") {
    body.prompt_mode = options.promptMode;
  }
  if (options.googleSearch) body.google_search = options.googleSearch;
  if (options.searchEngine) body.search_engine = options.searchEngine;
  if (customLLM) body.custom_llm = customLLM;

  return body;
}

function parseSseLine<T>(line: string, onData: (data: T) => void) {
  const normalizedLine = line.endsWith("\r") ? line.slice(0, -1) : line;
  if (!normalizedLine.startsWith("data:")) return;

  const payload = normalizedLine.slice(5).trimStart();
  if (!payload) return;

  onData(JSON.parse(payload) as T);
}

export async function consumeSseStream<T>(
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
      parseSseLine(line, onData);
    }
  }

  buffer += decoder.decode();
  if (buffer) {
    parseSseLine(buffer, onData);
  }
}
