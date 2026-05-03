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
  Card,
  DropdownMenu,
  Switch,
  TextArea,
  Tooltip,
  Flex,
  Text,
  Box,
} from "@radix-ui/themes";
import { useCallback, useEffect, useRef, useState } from "react";
import { useLocalStorage } from "usehooks-ts";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import {
  DiskIcon,
  listModel as listModels,
  Markdown,
  PlayIcon,
  SaveWordModal,
} from "../components";
import { PageHeader } from "@/ui/PageHeader";
import { PageContainer } from "@/ui/PageContainer";

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
const SOURCE_TAGS: Partial<Record<(typeof BUILTIN_METHODS)[number]["value"], string>> = {
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

const itemVariants = {
  hidden: { opacity: 0, y: 10 },
  visible: { opacity: 1, y: 0 },
};

export default function Home() {
  const [selected, setSelected] = useLocalStorage("translate_type", DEFAULT_METHOD);
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
  const [customModels, setCustomModels] = useLocalStorage<Record<string, CustomLLM>>(
    "custom_llms_cache",
    {},
  );
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
    <div className="app-page-shell min-h-dvh">
      <SaveWordModal
        open={open}
        onChange={(p) => setOpen(p)}
        word={query}
        explain={liveResult.result}
        type={0}
      />
      <PageContainer className="space-y-6">
        <PageHeader
          actions={
            <Flex
              gap="2"
              align="center"
              wrap="wrap"
              className="app-home-actions"
            >
              <Tooltip content="Translate (Ctrl+Enter)">
                <Button
                  variant="solid"
                  color="gray"
                  className="app-primary-action"
                  loading={loading}
                  onClick={doQueryWord}
                >
                  <Flex gap="2" align="center">
                    <PlayIcon size={18} />
                    Translate
                  </Flex>
                </Button>
              </Tooltip>

              <Tooltip content="Save Word (Ctrl+S)">
                <Button
                  variant="surface"
                  color="gray"
                  className="app-secondary-action"
                  onClick={() => setOpen(true)}
                >
                  <Flex gap="2" align="center">
                    <DiskIcon size={18} />
                    Save
                  </Flex>
                </Button>
              </Tooltip>
            </Flex>
          }
        >
          <div className="app-home-toolbar">
            <Text size="1" color="gray" className="app-toolbar-hint">
              Ctrl+Enter to translate. Ctrl+S to save.
            </Text>

            <Flex
              gap="2"
              align="center"
              wrap="wrap"
              className="app-control-row"
            >
              <DropdownMenu.Root modal={false}>
                <Tooltip content="Translate Method">
                  <DropdownMenu.Trigger>
                    <Button
                      variant="surface"
                      color="gray"
                      className="app-control-trigger capitalize cursor-pointer"
                    >
                      {selectedLabel || "Select"}
                    </Button>
                  </DropdownMenu.Trigger>
                </Tooltip>
                <DropdownMenu.Content>
                  <DropdownMenu.Group>
                    {translationSources.map((source) => (
                      <DropdownMenu.Item
                        key={source.key}
                        onSelect={() => setSelected(source.key)}
                      >
                        {source.name}
                      </DropdownMenu.Item>
                    ))}
                  </DropdownMenu.Group>

                  {dictSources.length > 0 && <DropdownMenu.Separator />}

                  <DropdownMenu.Group>
                    {dictSources.map((source) => (
                      <DropdownMenu.Item
                        key={source.key}
                        onSelect={() => setSelected(source.key)}
                      >
                        <Flex justify="between" width="100%" gap="4">
                          <Text>{source.name}</Text>
                          {source.tag && (
                            <Text color="gray" size="1">
                              {source.tag}
                            </Text>
                          )}
                        </Flex>
                      </DropdownMenu.Item>
                    ))}
                  </DropdownMenu.Group>

                  {Object.keys(customModels || {}).length > 0 && (
                    <DropdownMenu.Separator />
                  )}

                  {Object.keys(customModels || {}).length > 0 && (
                    <DropdownMenu.Group>
                      {Object.keys(customModels || {}).map((key) => (
                        <DropdownMenu.Item
                          key={key}
                          onSelect={() => setSelected(key)}
                        >
                          <Flex justify="between" width="100%" gap="4">
                            <Text>{customModels[key].model}</Text>
                            <Text color="gray" size="1">
                              {customModels[key].name}
                            </Text>
                          </Flex>
                        </DropdownMenu.Item>
                      ))}
                    </DropdownMenu.Group>
                  )}
                </DropdownMenu.Content>
              </DropdownMenu.Root>

              {showSelectLang(selected) && (
                <>
                  <DropdownMenu.Root modal={false}>
                    <Tooltip content="Source Language">
                      <DropdownMenu.Trigger>
                        <Button
                          variant="surface"
                          color="gray"
                          className="app-control-trigger cursor-pointer"
                        >
                          <span className="app-language-pill">
                            <span
                              className="app-language-flag"
                              aria-hidden="true"
                            >
                              {languageMap[srcLang]?.flag || "🌐"}
                            </span>
                            <span>{languageMap[srcLang]?.name || "Auto"}</span>
                          </span>
                        </Button>
                      </DropdownMenu.Trigger>
                    </Tooltip>
                    <DropdownMenu.Content>
                      {languages.map((lang) => (
                        <DropdownMenu.Item
                          key={lang.key}
                          onSelect={() => setSrcLang(lang.key)}
                        >
                          <Flex gap="2" align="center">
                            <span
                              className="app-language-flag"
                              aria-hidden="true"
                            >
                              {lang.flag}
                            </span>
                            <span>{lang.name}</span>
                          </Flex>
                        </DropdownMenu.Item>
                      ))}
                    </DropdownMenu.Content>
                  </DropdownMenu.Root>

                  <DropdownMenu.Root modal={false}>
                    <Tooltip content="Target Language">
                      <DropdownMenu.Trigger>
                        <Button
                          variant="surface"
                          color="gray"
                          className="app-control-trigger cursor-pointer"
                        >
                          <span className="app-language-pill">
                            <span
                              className="app-language-flag"
                              aria-hidden="true"
                            >
                              {languageMap[dstLang]?.flag || "🌐"}
                            </span>
                            <span>{languageMap[dstLang]?.name || "Auto"}</span>
                          </span>
                        </Button>
                      </DropdownMenu.Trigger>
                    </Tooltip>
                    <DropdownMenu.Content>
                      {languages
                        .filter((lang) => lang.key !== "")
                        .map((lang) => (
                          <DropdownMenu.Item
                            key={lang.key}
                            onSelect={() => setDstLang(lang.key)}
                          >
                            <Flex gap="2" align="center">
                              <span
                                className="app-language-flag"
                                aria-hidden="true"
                              >
                                {lang.flag}
                              </span>
                              <span>{lang.name}</span>
                            </Flex>
                          </DropdownMenu.Item>
                        ))}
                    </DropdownMenu.Content>
                  </DropdownMenu.Root>
                </>
              )}
            </Flex>
          </div>
        </PageHeader>

        <motion.div
          variants={shouldReduceMotion ? undefined : itemVariants}
          initial={shouldReduceMotion ? false : "hidden"}
          animate={shouldReduceMotion ? undefined : "visible"}
        >
          <Card className="app-section-card">
            <Box>
              <Text
                as="label"
                size="1"
                weight="bold"
                color="gray"
                className="mb-2 block uppercase tracking-widest"
              >
                Input
              </Text>
              <TextArea
                ref={queryInputRef}
                color={query.length === 0 ? "red" : undefined}
                value={query}
                placeholder="Type or paste text to translate..."
                variant="soft"
                className="app-textarea"
                onChange={(e) => setQuery(e.target.value)}
              />
            </Box>
          </Card>
        </motion.div>

        {hasCustomSelection && (
          <Card className="app-section-card">
            <Flex justify="between" align="start" gap="4" wrap="wrap">
              <Box className="min-w-0 space-y-1">
                <Text size="2" weight="bold" className="block leading-tight">
                  Advanced
                </Text>
                <Text
                  size="1"
                  color="gray"
                  className="block max-w-[28rem] leading-relaxed"
                >
                  Options for custom LLM queries
                </Text>
              </Box>
              <Button
                variant="soft"
                color="gray"
                className="cursor-pointer shrink-0"
                onClick={() => setShowAdvanced((v) => !v)}
              >
                {showAdvanced ? "Hide" : "Show"}
              </Button>
            </Flex>

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
}
