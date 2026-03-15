import { authorizedRequest, streamRequest } from "@/lib/api";
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
import { useCallback, useEffect, useState } from "react";
import { useLocalStorage } from "usehooks-ts";
import { motion, AnimatePresence } from "framer-motion";
import {
  DiskIcon,
  listModel as listModels,
  Markdown,
  PlayIcon,
  SaveWordModal,
  TTSButton,
} from "../components";
import { PageHeader } from "@/ui/PageHeader";
import { PageContainer } from "@/ui/PageContainer";

async function fetchTranslation(
  opts: {
    selected: string;
    query: string;
    instruction: string;
    google_search: boolean;
    srcLang: string;
    dstLang: string;
    custom_llm?: { name: string; model: string };
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
    body: JSON.stringify({
      method: opts.selected,
      word: opts.query,
      instruction: opts.instruction.length > 0 ? opts.instruction : undefined,
      google_search: opts.google_search,
      src_lang: opts.srcLang ? opts.srcLang : undefined,
      dst_lang: opts.dstLang ? opts.dstLang : undefined,
      custom_llm: opts.custom_llm,
    }),
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
  if (selected.startsWith("custom-")) {
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

const languages = [
  { key: "", name: "Auto", icon: "un" },
  { key: "ja", name: "Japanese", icon: "jp" },
  { key: "zh", name: "Chinese", icon: "cn" },
  { key: "en", name: "English", icon: "us" },
  { key: "ko", name: "Korean", icon: "kr" },
];

const languageMap = Object.fromEntries(
  languages.map(({ key, name, icon }) => [key, { name, icon }]),
) as Record<(typeof languages)[number]["key"], { name: string; icon: string }>;

type TranslationSource = {
  key: string;
  name: string;
};

const translationSources: TranslationSource[] = [
  { key: "google", name: "Google Translate" },
  { key: "googlev1", name: "Google Translate(old API)" },
  { key: "m2m100_1_2b", name: "m2m100-1.2b" },
];

const dictSources = [
  { key: "weblio", name: "Weblio" },
  { key: "ktbk", name: "コトバンク" },
  { key: "jc", name: "Japanese -> Chinese", tag: "hujiang" },
  { key: "cj", name: "Japanese <- Chinese", tag: "hujiang" },
  { key: "kr", name: "Korean <-> Chinese", tag: "hujiang" },
  { key: "en", name: "English <-> Chinese", tag: "hujiang" },
];

const translationMap = Object.fromEntries(
  [...translationSources, ...dictSources].map(({ key, name }) => [key, name]),
) as Record<(typeof translationSources)[number]["key"], string>;

const itemVariants = {
  hidden: { opacity: 0, y: 10 },
  visible: { opacity: 1, y: 0 },
};

export default function Home() {
  const [selected, setSelected] = useLocalStorage("translate_type", "ktbk");
  const [query, setQuery] = useLocalStorage("query", "");
  const [instruction, setInstruction] = useLocalStorage("instruction", "");
  const [googleSearch, setGoogleSearch] = useLocalStorage(
    "google_search",
    false,
  );
  const [result, setResult] = useLocalStorage<{
    result: string;
    reasoning?: string;
  }>("result_v2", { result: "" });
  const [srcLang, setSrcLang] = useLocalStorage("src_lang", "");
  const [dstLang, setDstLang] = useLocalStorage("dst_lang", "ja");
  const [loading, setLoading] = useState(false);
  const [stream, setStream] = useLocalStorage("stream", true);
  const [open, setOpen] = useState(false);
  const [showAdvanced, setShowAdvanced] = useLocalStorage(
    "home_advanced_open",
    false,
  );
  const [customModels, setCustomModels] = useLocalStorage<
    Record<string, { name: string; model: string }>
  >("custom_llms_cache", {});

  useEffect(() => {
    listModels((models, error) => {
      if (error) {
        console.log(error);
      } else if (models) {
        const customModelsMap: Record<string, { name: string; model: string }> =
          {};
        for (const llm of models) {
          for (const model of llm.models) {
            customModelsMap[`custom-${llm.name}-${model}`] = {
              name: llm.name,
              model: model,
            };
          }
        }
        setCustomModels(customModelsMap);
      }
    });
  }, [setCustomModels]);

  const doQueryWord = useCallback(async () => {
    if (!query) return;
    let modelName = selected;
    let customLLM: { name: string; model: string } | undefined;
    if (modelName.startsWith("custom-")) {
      customLLM = customModels?.[modelName];
      modelName = "custom_llm";
    }

    setLoading(true);

    if (stream && modelName === "custom_llm") {
      setResult({ result: "" });
      try {
        const resp = await authorizedRequest("/word/query_stream", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            method: modelName,
            word: query,
            instruction: instruction.length > 0 ? instruction : undefined,
            google_search: googleSearch,
            src_lang: srcLang ? srcLang : undefined,
            dst_lang: dstLang ? dstLang : undefined,
            custom_llm: customLLM,
          }),
        });

        if (!resp.ok) {
          setResult({ result: `(${resp.status}) ${await resp.text()}` });
          setLoading(false);
          return;
        }

        await streamRequest<{ result: string; reasoning?: string }>(
          resp,
          (data) => {
            setResult((prev) => ({
              result: prev.result + (data.result || ""),
              reasoning: (prev.reasoning || "") + (data.reasoning || ""),
            }));
          },
        );
      } catch (error) {
        setResult({ result: String(error) });
      } finally {
        setLoading(false);
      }
    } else {
      await fetchTranslation(
        {
          query: query,
          srcLang: srcLang,
          dstLang: dstLang,
          google_search: googleSearch,
          instruction: instruction,
          selected: modelName,
          custom_llm: customLLM,
        },
        (data, error) => {
          console.log(data);
          if (error) {
            setResult({ result: error });
          } else if (data) {
            setResult(data);
          } else {
            setResult({ result: "NOT FOUND" });
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
    instruction,
    selected,
    customModels,
    setResult,
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

  return (
    <div className="min-h-dvh">
      <SaveWordModal
        open={open}
        onChange={(p) => setOpen(p)}
        word={query}
        explain={result.result}
        type={0}
      />
      <PageContainer className="space-y-6">
        <PageHeader
          title="HJ Dict"
          subtitle="Ctrl+Enter to translate · Ctrl+S to save"
          actions={
            <Flex gap="2" align="center" wrap="wrap">
              <Tooltip content="Translate (Ctrl+Enter)">
                <Button
                  variant="solid"
                  color="blue"
                  className="cursor-pointer"
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
                  color="green"
                  className="cursor-pointer"
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
          <Flex gap="2" align="center" wrap="wrap" justify="between">
            <DropdownMenu.Root>
              <Tooltip content="Translate Method">
                <DropdownMenu.Trigger>
                  <Button
                    variant="surface"
                    color="gray"
                    className="capitalize cursor-pointer"
                  >
                    {translationMap[selected] ||
                      customModels?.[selected]?.model ||
                      "Select"}
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
              <Flex gap="2" align="center" wrap="wrap">
                <DropdownMenu.Root>
                  <Tooltip content="Source Language">
                    <DropdownMenu.Trigger>
                      <Button
                        variant="surface"
                        color="gray"
                        className="cursor-pointer"
                      >
                        {languageMap[srcLang]?.name || "Auto"}
                      </Button>
                    </DropdownMenu.Trigger>
                  </Tooltip>
                  <DropdownMenu.Content>
                    {languages.map((lang) => (
                      <DropdownMenu.Item
                        key={lang.key}
                        onSelect={() => setSrcLang(lang.key)}
                      >
                        {lang.name}
                      </DropdownMenu.Item>
                    ))}
                  </DropdownMenu.Content>
                </DropdownMenu.Root>

                <DropdownMenu.Root>
                  <Tooltip content="Target Language">
                    <DropdownMenu.Trigger>
                      <Button
                        variant="surface"
                        color="gray"
                        className="cursor-pointer"
                      >
                        {languageMap[dstLang]?.name || "Auto"}
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
                          {lang.name}
                        </DropdownMenu.Item>
                      ))}
                  </DropdownMenu.Content>
                </DropdownMenu.Root>
              </Flex>
            )}
          </Flex>
        </PageHeader>

        <motion.div variants={itemVariants} initial="hidden" animate="visible">
          <Card>
            <Box>
              <Flex justify="between" align="center" className="mb-2">
                <Text
                  as="label"
                  size="1"
                  weight="bold"
                  color="gray"
                  className="block uppercase tracking-widest"
                >
                  Input
                </Text>
                <TTSButton text={query} lang={srcLang || "en"} />
              </Flex>
              <TextArea
                color={query.length === 0 ? "red" : undefined}
                value={query}
                placeholder="Type or paste text to translate..."
                variant="soft"
                className="min-h-[80px]"
                onChange={(e) => setQuery(e.target.value)}
              />
            </Box>
          </Card>
        </motion.div>

        {selected.startsWith("custom-") && (
          <Card>
            <Flex justify="between" align="center" gap="3" wrap="wrap">
              <Box>
                <Text size="2" weight="bold">
                  Advanced
                </Text>
                <Text size="1" color="gray">
                  Options for custom LLM queries
                </Text>
              </Box>
              <Button
                variant="soft"
                color="gray"
                className="cursor-pointer"
                onClick={() => setShowAdvanced((v) => !v)}
              >
                {showAdvanced ? "Hide" : "Show"}
              </Button>
            </Flex>

            <AnimatePresence initial={false}>
              {showAdvanced && (
                <motion.div
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: "auto" }}
                  exit={{ opacity: 0, height: 0 }}
                  transition={{ duration: 0.25 }}
                  className="mt-4 space-y-4 overflow-hidden"
                >
                  <Flex gap="6" wrap="wrap">
                    <Text as="label" size="2" weight="medium">
                      <Flex gap="2" align="center" className="cursor-pointer">
                        <Switch
                          checked={googleSearch}
                          onCheckedChange={(e) => setGoogleSearch(e)}
                        />
                        Google Search
                      </Flex>
                    </Text>

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
                        className="min-h-[60px]"
                        onChange={(e) => setInstruction(e.target.value)}
                      />
                    </Box>
                  )}
                </motion.div>
              )}
            </AnimatePresence>
          </Card>
        )}

        <AnimatePresence mode="wait">
          {result.reasoning && (
            <motion.div
              key="reasoning"
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
            >
              <Text
                size="1"
                weight="bold"
                color="blue"
                className="mb-2 ml-1 block uppercase tracking-widest"
              >
                Thinking Process
              </Text>
              <Card variant="surface">
                <Box className="prose prose-sm dark:prose-invert max-w-none opacity-80">
                  <Markdown>{result.reasoning}</Markdown>
                </Box>
              </Card>
            </motion.div>
          )}
        </AnimatePresence>

        <AnimatePresence mode="wait">
          <motion.div
            key="result"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
          >
            <Flex justify="between" align="center" className="mb-2 ml-1">
              <Text
                size="1"
                weight="bold"
                color="gray"
                className="block uppercase tracking-widest"
              >
                Result
              </Text>
              <TTSButton text={result.result} lang={dstLang || "en"} />
            </Flex>
            <Card>
              {result.result ? (
                <Box className="prose prose-sm dark:prose-invert max-w-none">
                  <Markdown>{result.result}</Markdown>
                </Box>
              ) : (
                <Flex
                  className="min-h-[150px]"
                  align="center"
                  justify="center"
                  direction="column"
                  gap="3"
                >
                  <motion.div
                    animate={{
                      scale: [1, 1.08, 1],
                      opacity: [0.15, 0.3, 0.15],
                    }}
                    transition={{
                      repeat: Infinity,
                      duration: 3,
                      ease: "easeInOut",
                    }}
                  >
                    <PlayIcon size={40} />
                  </motion.div>
                  <Text color="gray" size="2" className="italic">
                    Waiting for input...
                  </Text>
                </Flex>
              )}
            </Card>
          </motion.div>
        </AnimatePresence>
      </PageContainer>
    </div>
  );
}
