import { authorizedRequest, streamRequest } from "@/lib/api";
import {
  BUILTIN_METHODS,
  DEFAULT_METHOD,
  DEFAULT_SEARCH_ENGINE,
  LANGUAGES,
  PROMPT_MODES,
  SEARCH_ENGINES,
  buildTranslationQueryBody,
  customMethodValue,
  legacyCustomMethodValue,
  methodLabel,
  parseCustomMethod,
  type CustomLLM,
  type PromptMode,
} from "@/lib/translation";
import {
  Button,
  DropdownMenu,
  Switch,
  TextArea,
} from "@radix-ui/themes";
import { useCallback, useEffect, useRef, useState } from "react";
import { useLocalStorage } from "usehooks-ts";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { ROUTE_FLASHCARD } from "@/lib/constants";
import {
  listModel as listModels,
  Markdown,
  SaveWordModal,
} from "../components";
import {
  ArrowUp,
  ChevronDown,
  Globe2,
  Layers3,
  PencilLine,
  Plus,
  Search,
  SlidersHorizontal,
  Sparkles,
} from "lucide-react";
import { useLocation } from "wouter";

async function fetchTranslation(
  opts: {
    method: string;
    query: string;
    instruction: string;
    promptMode?: PromptMode;
    googleSearch: boolean;
    searchEngine?: string;
    srcLang: string;
    dstLang: string;
  },
  callback: (
    data?: { result: string; reasoning?: string },
    error?: string,
  ) => void,
) {
  const resp = await authorizedRequest("/word/query", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(
      buildTranslationQueryBody({
        method: opts.method,
        text: opts.query,
        instruction: opts.instruction.length > 0 ? opts.instruction : undefined,
        promptMode: opts.promptMode,
        googleSearch: opts.googleSearch,
        searchEngine: opts.searchEngine ? opts.searchEngine : undefined,
        srcLang: opts.srcLang ? opts.srcLang : undefined,
        dstLang: opts.dstLang ? opts.dstLang : undefined,
      }),
    ),
  });

  if (resp.ok) {
    callback(
      (await resp.json()) as { result: string; reasoning?: string },
      undefined,
    );
  } else {
    callback(undefined, `(${resp.status}) ${await resp.text()}`);
  }
}

function showSelectLang(selected: string) {
  if (selected.startsWith("custom-") || parseCustomMethod(selected)) {
    return true;
  }

  switch (selected) {
    case "google":
    case "googlev1":
    case "m2m100_1_2b":
      return true;
    default:
      return false;
  }
}

const LANGUAGE_FLAGS: Record<string, string> = {
  "": "🌐",
  ja: "🇯🇵",
  zh: "🇨🇳",
  "zh-CN": "🇨🇳",
  "zh-TW": "🇹🇼",
  en: "🇺🇸",
  ko: "🇰🇷",
  fr: "🇫🇷",
  de: "🇩🇪",
  es: "🇪🇸",
  it: "🇮🇹",
  pt: "🇵🇹",
  ru: "🇷🇺",
  vi: "🇻🇳",
  th: "🇹🇭",
  id: "🇮🇩",
};

const languages = LANGUAGES.map(({ value, label }) => ({
  key: value,
  name: label,
  flag: LANGUAGE_FLAGS[value] ?? "🌐",
}));

const languageMap = Object.fromEntries(
  languages.map(({ key, name, flag }) => [key, { name, flag }]),
);

const TRANSLATION_SOURCE_KEYS = new Set(["google", "googlev1", "m2m100_1_2b"]);
const SOURCE_TAGS: Partial<
  Record<(typeof BUILTIN_METHODS)[number]["value"], string>
> = {
  jc: "hujiang",
  cj: "hujiang",
  kr: "hujiang",
  en: "hujiang",
};

const translationSources = BUILTIN_METHODS.filter((method) =>
  TRANSLATION_SOURCE_KEYS.has(method.value),
).map((method) => ({
  key: method.value,
  name: method.label,
}));

const dictSources = BUILTIN_METHODS.filter(
  (method) => !TRANSLATION_SOURCE_KEYS.has(method.value),
).map((method) => ({
  key: method.value,
  name: method.label,
  tag: SOURCE_TAGS[method.value],
}));

const translationMap = Object.fromEntries(
  BUILTIN_METHODS.map((method) => [method.value, method.label]),
);

const promptModeLabels = Object.fromEntries(
  PROMPT_MODES.map((mode) => [mode.value, mode.label]),
) as Record<PromptMode, string>;

function resolveCustomModel(
  value: string,
  customModels: Record<string, CustomLLM>,
): CustomLLM | undefined {
  if (customModels[value]) {
    return customModels[value];
  }

  const normalized = parseCustomMethod(value);
  if (normalized) {
    return normalized;
  }

  if (!value.startsWith("custom-")) {
    return undefined;
  }

  return Object.values(customModels).find(
    (customModel) => legacyCustomMethodValue(customModel) === value,
  );
}

export default function Home() {
  const [, setLocation] = useLocation();
  const [selected, setSelected] = useLocalStorage(
    "translate_type",
    DEFAULT_METHOD,
  );
  const [query, setQuery] = useLocalStorage("query", "");
  const [instruction, setInstruction] = useLocalStorage("instruction", "");
  const [googleSearch, setGoogleSearch] = useLocalStorage(
    "google_search",
    false,
  );
  const [searchEngine, setSearchEngine] = useLocalStorage<string>(
    "search_engine",
    DEFAULT_SEARCH_ENGINE,
  );
  const [result, setResult] = useLocalStorage<{
    result: string;
    reasoning?: string;
  }>("result_v2", { result: "" });
  const [liveResult, setLiveResult] = useState(result);
  const [srcLang, setSrcLang] = useLocalStorage("src_lang", "");
  const [dstLang, setDstLang] = useLocalStorage("dst_lang", "ja");
  const [loading, setLoading] = useState(false);
  const [stream, setStream] = useLocalStorage("stream", true);
  const [promptMode, setPromptMode] = useLocalStorage<PromptMode>(
    "prompt_mode",
    "default",
  );
  const [open, setOpen] = useState(false);
  const [showAdvanced, setShowAdvanced] = useLocalStorage(
    "home_advanced_open",
    false,
  );
  const [customModels, setCustomModels] = useLocalStorage<
    Record<string, CustomLLM>
  >("custom_llms_cache", {});
  const shouldReduceMotion = useReducedMotion();
  const streamFrameRef = useRef<number | null>(null);
  const persistTimeoutRef = useRef<number | null>(null);
  const pendingStreamRef = useRef({ result: "", reasoning: "" });
  const queryInputRef = useRef<HTMLTextAreaElement | null>(null);

  const persistResult = useCallback(
    (
      next:
        | { result: string; reasoning?: string }
        | ((prev: { result: string; reasoning?: string }) => {
            result: string;
            reasoning?: string;
          }),
      options?: { debounce?: boolean; delay?: number },
    ) => {
      setLiveResult((prev) => {
        const nextValue = typeof next === "function" ? next(prev) : next;

        if (persistTimeoutRef.current !== null) {
          window.clearTimeout(persistTimeoutRef.current);
        }

        if (options?.debounce) {
          persistTimeoutRef.current = window.setTimeout(() => {
            setResult(nextValue);
            persistTimeoutRef.current = null;
          }, options.delay ?? 260);
        } else {
          setResult(nextValue);
        }

        return nextValue;
      });
    },
    [setResult],
  );

  useEffect(() => {
    return () => {
      if (streamFrameRef.current !== null) {
        window.cancelAnimationFrame(streamFrameRef.current);
      }
      if (persistTimeoutRef.current !== null) {
        window.clearTimeout(persistTimeoutRef.current);
      }
    };
  }, []);

  useEffect(() => {
    const textarea = queryInputRef.current;
    if (!textarea) return;
    textarea.style.height = "0px";
    textarea.style.height = `${Math.max(120, textarea.scrollHeight)}px`;
  }, [query]);

  useEffect(() => {
    listModels((models, error) => {
      if (error) {
        console.log(error);
      } else if (models) {
        const customModelsMap: Record<string, CustomLLM> = {};
        for (const llm of models) {
          for (const model of llm.models) {
            const value = customMethodValue({ name: llm.name, model });
            customModelsMap[value] = {
              name: llm.name,
              model,
            };
          }
        }
        setCustomModels(customModelsMap);
      }
    });
  }, [setCustomModels]);

  useEffect(() => {
    if (!selected.startsWith("custom-")) return;

    const customModel = resolveCustomModel(selected, customModels);
    if (!customModel) return;

    const normalizedValue = customMethodValue(customModel);
    if (normalizedValue !== selected) {
      setSelected(normalizedValue);
    }
  }, [customModels, selected, setSelected]);

  const doQueryWord = useCallback(async () => {
    if (!query) return;
    const customLLM = resolveCustomModel(selected, customModels);
    const method = customLLM ? customMethodValue(customLLM) : selected;

    setLoading(true);

    if (stream && customLLM) {
      pendingStreamRef.current = { result: "", reasoning: "" };
      persistResult({ result: "" });
      try {
        const resp = await authorizedRequest("/word/query_stream", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(
            buildTranslationQueryBody({
              method,
              text: query,
              instruction: instruction.length > 0 ? instruction : undefined,
              promptMode,
              googleSearch,
              searchEngine: searchEngine ? searchEngine : undefined,
              srcLang: srcLang ? srcLang : undefined,
              dstLang: dstLang ? dstLang : undefined,
            }),
          ),
        });

        if (!resp.ok) {
          persistResult({ result: `(${resp.status}) ${await resp.text()}` });
          setLoading(false);
          return;
        }

        await streamRequest<{ result: string; reasoning?: string }>(
          resp,
          (data) => {
            pendingStreamRef.current = {
              result: pendingStreamRef.current.result + (data.result || ""),
              reasoning:
                pendingStreamRef.current.reasoning + (data.reasoning || ""),
            };

            if (streamFrameRef.current !== null) return;

            streamFrameRef.current = window.requestAnimationFrame(() => {
              streamFrameRef.current = null;
              const pending = pendingStreamRef.current;
              pendingStreamRef.current = { result: "", reasoning: "" };

              persistResult(
                (prev) => ({
                  result: prev.result + pending.result,
                  reasoning: (prev.reasoning || "") + pending.reasoning,
                }),
                { debounce: true },
              );
            });
          },
        );
      } catch (error) {
        persistResult({ result: String(error) });
      } finally {
        setLoading(false);
      }
    } else {
      await fetchTranslation(
        {
          query: query,
          srcLang: srcLang,
          dstLang: dstLang,
          googleSearch,
          searchEngine,
          instruction: instruction,
          promptMode,
          method,
        },
        (data, error) => {
          console.log(data);
          if (error) {
            persistResult({ result: error });
          } else if (data) {
            persistResult(data);
          } else {
            persistResult({ result: "NOT FOUND" });
          }
          setLoading(false);
        },
      );
    }
  }, [
    query,
    srcLang,
    dstLang,
    googleSearch,
    searchEngine,
    instruction,
    promptMode,
    selected,
    customModels,
    persistResult,
    setLoading,
    stream,
  ]);

  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      // Windows/Linux: Ctrl, macOS: Meta(Command)
      if (e.ctrlKey || e.metaKey) {
        switch (e.key) {
          case "s":
            e.preventDefault();
            setOpen(true);
            break;

          case "Enter":
            e.preventDefault();
            await doQueryWord();
            break;
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [setOpen, doQueryWord]);

  const selectedCustomModel = resolveCustomModel(selected, customModels);
  const selectedLabel = selectedCustomModel
    ? selectedCustomModel.model
    : translationMap[selected] || methodLabel(selected);
  const hasCustomSelection = Boolean(selectedCustomModel);

  return (
    <div className="app-home-page">
      <SaveWordModal
        open={open}
        onChange={(p) => setOpen(p)}
        word={query}
        explain={liveResult.result}
        type={0}
      />
      <section className="app-home-stage">
        <div className="app-home-kicker"><Sparkles size={15} /> Hujiang dictionary</div>
        <h1>What can I help you find?</h1>
        <p className="app-home-subtitle">Translate, understand, and save language in one place.</p>

        <div className="app-composer">
          <div className="app-composer-input-row">
            <button type="button" className="app-composer-add" aria-label="Add context">
              <Plus size={21} />
            </button>
            <TextArea
              ref={queryInputRef}
              value={query}
              placeholder="Ask DictDeck anything about a word or phrase"
              variant="soft"
              className="app-composer-textarea"
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>

          <div className="app-composer-footer">
            <div className="app-composer-controls">
              <DropdownMenu.Root modal={false}>
                <DropdownMenu.Trigger>
                  <Button variant="ghost" className="app-composer-control">
                    <Sparkles size={16} />
                    <span>{selectedLabel || "Select source"}</span>
                    <ChevronDown size={14} />
                  </Button>
                </DropdownMenu.Trigger>
                <DropdownMenu.Content>
                  <DropdownMenu.Label>Translation source</DropdownMenu.Label>
                  {translationSources.map((source) => (
                    <DropdownMenu.Item key={source.key} onSelect={() => setSelected(source.key)}>
                      {source.name}
                    </DropdownMenu.Item>
                  ))}
                  <DropdownMenu.Separator />
                  {dictSources.map((source) => (
                    <DropdownMenu.Item key={source.key} onSelect={() => setSelected(source.key)}>
                      {source.name}
                    </DropdownMenu.Item>
                  ))}
                  {Object.keys(customModels || {}).length > 0 && <DropdownMenu.Separator />}
                  {Object.keys(customModels || {}).map((key) => (
                    <DropdownMenu.Item key={key} onSelect={() => setSelected(key)}>
                      {customModels[key].model}
                    </DropdownMenu.Item>
                  ))}
                </DropdownMenu.Content>
              </DropdownMenu.Root>

              {showSelectLang(selected) && (
                <>
                  <DropdownMenu.Root modal={false}>
                    <DropdownMenu.Trigger>
                      <Button variant="ghost" className="app-composer-control app-composer-language">
                        <span>{languageMap[srcLang]?.flag || "🌐"}</span>
                        <span>{languageMap[srcLang]?.name || "Auto"}</span>
                      </Button>
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content>
                      {languages.map((lang) => (
                        <DropdownMenu.Item key={lang.key} onSelect={() => setSrcLang(lang.key)}>
                          {lang.flag} {lang.name}
                        </DropdownMenu.Item>
                      ))}
                    </DropdownMenu.Content>
                  </DropdownMenu.Root>
                  <DropdownMenu.Root modal={false}>
                    <DropdownMenu.Trigger>
                      <Button variant="ghost" className="app-composer-control app-composer-language">
                        <span>{languageMap[dstLang]?.flag || "🌐"}</span>
                        <span>{languageMap[dstLang]?.name || "Auto"}</span>
                      </Button>
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content>
                      {languages.filter((lang) => lang.key !== "").map((lang) => (
                        <DropdownMenu.Item key={lang.key} onSelect={() => setDstLang(lang.key)}>
                          {lang.flag} {lang.name}
                        </DropdownMenu.Item>
                      ))}
                    </DropdownMenu.Content>
                  </DropdownMenu.Root>
                </>
              )}

              {hasCustomSelection && (
                <button
                  type="button"
                  className={`app-composer-control app-composer-icon-control${showAdvanced ? " is-active" : ""}`}
                  onClick={() => setShowAdvanced((value) => !value)}
                  aria-label="Toggle advanced options"
                >
                  <SlidersHorizontal size={16} />
                </button>
              )}
            </div>

            <div className="app-composer-submit-group">
              <span className="app-composer-shortcut">⌘ Enter</span>
              <button
                type="button"
                className="app-composer-submit"
                onClick={doQueryWord}
                disabled={loading || !query.trim()}
                aria-label="Translate"
              >
                {loading ? <span className="app-loading-dot" /> : <ArrowUp size={19} strokeWidth={2.5} />}
              </button>
            </div>
          </div>
        </div>

        <div className="app-quick-actions" aria-label="Quick actions">
          <button type="button" onClick={() => queryInputRef.current?.focus()}>
            <Search size={17} />
            Dictionary lookup
          </button>
          <button type="button" onClick={() => { setSelected("google"); queryInputRef.current?.focus(); }}>
            <Globe2 size={17} />
            Translate a sentence
          </button>
          <button type="button" onClick={() => setOpen(true)}>
            <PencilLine size={17} />
            Save a word
          </button>
          <button type="button" onClick={() => setLocation(ROUTE_FLASHCARD)}>
            <Layers3 size={17} />
            Review vocabulary
          </button>
        </div>
      </section>

        {hasCustomSelection && (
          <AnimatePresence initial={false}>
            {showAdvanced && (
              <motion.section
                className="app-advanced-panel"
                initial={shouldReduceMotion ? false : { opacity: 0, height: 0 }}
                animate={shouldReduceMotion ? undefined : { opacity: 1, height: "auto" }}
                exit={shouldReduceMotion ? undefined : { opacity: 0, height: 0 }}
              >
                <div className="app-advanced-heading">
                  <div>
                    <strong>Advanced options</strong>
                    <span>Fine-tune this lookup without leaving the composer.</span>
                  </div>
                  <button type="button" onClick={() => setShowAdvanced(false)}>Close</button>
                </div>
                <div className="app-advanced-grid">
                  <label className="app-toggle-row">
                    <span><Switch checked={googleSearch} onCheckedChange={setGoogleSearch} /> Web search</span>
                    <small>Use live sources when translating.</small>
                  </label>
                  <label className="app-toggle-row">
                    <span><Switch checked={stream} onCheckedChange={setStream} /> Stream response</span>
                    <small>Show the answer as it arrives.</small>
                  </label>
                  {googleSearch && (
                    <label className="app-field-stack">
                      <span>Search engine</span>
                      <select value={searchEngine} onChange={(event) => setSearchEngine(event.target.value)} className="app-native-select">
                        {SEARCH_ENGINES.map((engine) => <option key={engine.value} value={engine.value}>{engine.label}</option>)}
                      </select>
                    </label>
                  )}
                  <label className="app-field-stack">
                    <span>Prompt mode</span>
                    <DropdownMenu.Root modal={false}>
                      <DropdownMenu.Trigger>
                        <Button variant="ghost" className="app-advanced-select">{promptModeLabels[promptMode]} <ChevronDown size={14} /></Button>
                      </DropdownMenu.Trigger>
                      <DropdownMenu.Content>
                        {PROMPT_MODES.map((mode) => <DropdownMenu.Item key={mode.value} onSelect={() => setPromptMode(mode.value)}>{mode.label}</DropdownMenu.Item>)}
                      </DropdownMenu.Content>
                    </DropdownMenu.Root>
                  </label>
                  {!googleSearch && (
                    <label className="app-field-stack app-field-wide">
                      <span>Custom instruction</span>
                      <TextArea value={instruction} placeholder="Translate to natural spoken Japanese..." onChange={(event) => setInstruction(event.target.value)} />
                    </label>
                  )}
                </div>
              </motion.section>
            )}
          </AnimatePresence>
        )}

        {(liveResult.reasoning || liveResult.result) && (
          <section className="app-home-thread">
            {query && (
              <div className="app-thread-user">
                <div className="app-thread-avatar app-thread-avatar-user">You</div>
                <div className="app-thread-user-bubble">{query}</div>
              </div>
            )}
            {liveResult.reasoning && (
              <div className="app-thread-message app-thread-reasoning">
                <div className="app-thread-avatar">D</div>
                <div className="app-thread-message-body">
                  <div className="app-thread-label">Thinking process</div>
                  <div className="prose"><Markdown>{liveResult.reasoning}</Markdown></div>
                </div>
              </div>
            )}
            <div className="app-thread-message">
              <div className="app-thread-avatar">D</div>
              <div className="app-thread-message-body">
                <div className="app-thread-label">DictDeck</div>
                <div className="prose"><Markdown>{liveResult.result || (loading ? "Looking that up…" : "No result found.")}</Markdown></div>
                <div className="app-thread-actions">
                  <button type="button" onClick={() => setOpen(true)}>Save to vocabulary</button>
                  <button type="button" onClick={() => navigator.clipboard?.writeText(liveResult.result)}>Copy</button>
                </div>
              </div>
            </div>
          </section>
        )}

        {!liveResult.result && !liveResult.reasoning && (
          <section className="app-home-suggestions">
            <div className="app-home-suggestion-heading">Try one of these</div>
            <div className="app-home-suggestion-grid">
              <button type="button" onClick={() => { setQuery("一期一会"); queryInputRef.current?.focus(); }}>
                <span className="app-suggestion-icon"><Search size={17} /></span>
                <span><strong>一期一会</strong><small>Look up a Japanese phrase</small></span>
              </button>
              <button type="button" onClick={() => { setQuery("How do I say thank you naturally?"); queryInputRef.current?.focus(); }}>
                <span className="app-suggestion-icon"><Globe2 size={17} /></span>
                <span><strong>Natural translation</strong><small>Ask for a better way to say it</small></span>
              </button>
              <button type="button" onClick={() => setLocation(ROUTE_FLASHCARD)}>
                <span className="app-suggestion-icon"><Sparkles size={17} /></span>
                <span><strong>Start a review</strong><small>Practice saved vocabulary</small></span>
              </button>
            </div>
          </section>
        )}
      </div>
  );
  /*
  return (
    <div className="app-page-shell min-h-dvh">
            <AnimatePresence initial={false}>
              {showAdvanced && (
                <motion.div
                  initial={
                    shouldReduceMotion
                      ? false
                      : { opacity: 0, height: 0, y: -6 }
                  }
                  animate={
                    shouldReduceMotion
                      ? undefined
                      : { opacity: 1, height: "auto", y: 0 }
                  }
                  exit={
                    shouldReduceMotion
                      ? undefined
                      : { opacity: 0, height: 0, y: -6 }
                  }
                  transition={{ duration: 0.18, ease: "easeOut" }}
                  className="mt-4 space-y-4 overflow-hidden"
                >
                  <Flex gap="6" wrap="wrap" align="center">
                    <Text as="label" size="2" weight="medium">
                      <Flex gap="2" align="center" className="cursor-pointer">
                        <Switch
                          checked={googleSearch}
                          onCheckedChange={(e) => setGoogleSearch(e)}
                        />
                        Google Search
                      </Flex>
                    </Text>

                    {googleSearch && (
                      <Box className="relative">
                        <select
                          value={searchEngine}
                          onChange={(event) =>
                            setSearchEngine(event.target.value)
                          }
                          className="app-native-select pr-10"
                        >
                          {SEARCH_ENGINES.map((engine) => (
                            <option key={engine.value} value={engine.value}>
                              {engine.label}
                            </option>
                          ))}
                        </select>
                        <span className="pointer-events-none absolute inset-y-0 right-4 flex items-center text-[var(--app-muted)]">
                          ▾
                        </span>
                      </Box>
                    )}

                    <Text as="label" size="2" weight="medium">
                      <Flex gap="2" align="center" className="cursor-pointer">
                        <Switch
                          checked={stream}
                          onCheckedChange={(e) => setStream(e)}
                        />
                        Stream
                      </Flex>
                    </Text>
                  </Flex>

                  <Box>
                    <Text
                      as="label"
                      size="1"
                      weight="bold"
                      color="gray"
                      className="mb-2 block uppercase tracking-widest"
                    >
                      Prompt Mode
                    </Text>
                    <DropdownMenu.Root modal={false}>
                      <DropdownMenu.Trigger>
                        <Button
                          variant="surface"
                          color="gray"
                          className="cursor-pointer"
                        >
                          {promptModeLabels[promptMode]}
                        </Button>
                      </DropdownMenu.Trigger>
                      <DropdownMenu.Content>
                        {PROMPT_MODES.map((mode) => (
                          <DropdownMenu.Item
                            key={mode.value}
                            onSelect={() => setPromptMode(mode.value)}
                          >
                            {mode.label}
                          </DropdownMenu.Item>
                        ))}
                      </DropdownMenu.Content>
                    </DropdownMenu.Root>
                  </Box>

                  {!googleSearch && (
                    <Box>
                      <Text
                        as="label"
                        size="1"
                        weight="bold"
                        color="gray"
                        className="mb-2 block uppercase tracking-widest"
                      >
                        Custom Instruction
                      </Text>
                      <TextArea
                        value={instruction}
                        placeholder="e.g. Translate to natural spoken Japanese..."
                        className="app-textarea min-h-[60px]"
                        onChange={(e) => setInstruction(e.target.value)}
                      />
                    </Box>
                  )}
                </motion.div>
              )}
            </AnimatePresence>
          </Card>
        )}

        {liveResult.reasoning && (
          <motion.div
            initial={shouldReduceMotion ? false : { opacity: 0, y: 10 }}
            animate={shouldReduceMotion ? undefined : { opacity: 1, y: 0 }}
          >
            <Text
              size="1"
              weight="bold"
              color="blue"
              className="mb-2 ml-1 block uppercase tracking-widest"
            >
              Thinking Process
            </Text>
            <Card variant="surface" className="app-section-card">
              <Box className="prose prose-sm dark:prose-invert max-w-none opacity-80">
                <Markdown>{liveResult.reasoning}</Markdown>
              </Box>
            </Card>
          </motion.div>
        )}

        <motion.div
          initial={shouldReduceMotion ? false : { opacity: 0, y: 10 }}
          animate={shouldReduceMotion ? undefined : { opacity: 1, y: 0 }}
        >
          <Flex
            justify="between"
            align="center"
            wrap="wrap"
            gap="3"
            className="mb-2 px-1"
          >
            <Text
              size="1"
              weight="bold"
              color="gray"
              className="block uppercase tracking-widest"
            >
              Result
            </Text>
            <div className="app-stat-chip text-xs">
              {loading
                ? "Translating..."
                : `${hasCustomSelection ? "Custom LLM" : "Dictionary"} output`}
            </div>
          </Flex>
          <Card className="app-section-card">
            {liveResult.result ? (
              <Box className="prose prose-sm dark:prose-invert max-w-none">
                <Markdown>{liveResult.result}</Markdown>
              </Box>
            ) : (
              <Flex
                className="min-h-[150px]"
                align="center"
                justify="center"
                direction="column"
                gap="3"
              >
                <div className="rounded-full border border-[var(--gray-a6)] bg-[var(--gray-a2)] p-4 text-[var(--gray-a10)]">
                  <PlayIcon size={34} />
                </div>
                <Text color="gray" size="2" className="italic">
                  Waiting for input...
                </Text>
              </Flex>
            )}
          </Card>
        </motion.div>
      </PageContainer>
    </div>
  );
  */
}
