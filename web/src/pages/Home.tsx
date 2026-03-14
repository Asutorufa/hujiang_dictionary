import { authorizedRequest, streamRequest } from "@/lib/api";
import {
  Avatar,
  Button,
  Card,
  DropdownMenu,
  Switch,
  TextArea,
  Tooltip,
  IconButton,
  Flex,
  Text,
  Box,
} from "@radix-ui/themes";
import { useCallback, useEffect, useState } from "react";
import { useLocalStorage } from "usehooks-ts";
import {
  DiskIcon,
  listModel as listModels,
  Markdown,
  PlayIcon,
  SaveWordModal,
} from "../components";

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
    <>
      <SaveWordModal
        open={open}
        onChange={(p) => setOpen(p)}
        word={query}
        explain={result.result}
        type={0}
      />
      <div className="p-2">
        <div className="sticky flex flex-wrap justify-center top-1 z-50 gap-1">
          <div className="flex gap-1">
            <DropdownMenu.Root>
              <Tooltip content="Translate Method">
                <DropdownMenu.Trigger>
                  <Button
                    variant="surface"
                    color="gray"
                    className="shadow-md backdrop-blur-sm capitalize"
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
                      <Flex justify="between" width="100%">
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
                        <Flex justify="between" width="100%">
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
                <DropdownMenu.Root>
                  <Tooltip content="Source Language">
                    <DropdownMenu.Trigger>
                      <IconButton
                        variant="surface"
                        color="gray"
                        className="shadow-md backdrop-blur-sm"
                      >
                        <Avatar
                          fallback="Auto"
                          alt={languageMap[srcLang]?.name || "Auto"}
                          size="1"
                          src={
                            languageMap[srcLang]?.icon
                              ? `https://flagcdn.com/${languageMap[srcLang].icon}.svg`
                              : undefined
                          }
                        />
                      </IconButton>
                    </DropdownMenu.Trigger>
                  </Tooltip>
                  <DropdownMenu.Content>
                    {languages.map((lang) => (
                      <DropdownMenu.Item
                        key={lang.key}
                        onSelect={() => setSrcLang(lang.key)}
                      >
                        <Flex gap="2" align="center">
                          {lang.icon && (
                            <Avatar
                              fallback={lang.name.charAt(0)}
                              alt={lang.name}
                              size="1"
                              src={`https://flagcdn.com/${lang.icon}.svg`}
                            />
                          )}
                          {lang.name}
                        </Flex>
                      </DropdownMenu.Item>
                    ))}
                  </DropdownMenu.Content>
                </DropdownMenu.Root>

                <DropdownMenu.Root>
                  <Tooltip content="Target Language">
                    <DropdownMenu.Trigger>
                      <IconButton
                        variant="surface"
                        color="gray"
                        className="shadow-md backdrop-blur-sm"
                      >
                        <Avatar
                          fallback="Auto"
                          alt={languageMap[dstLang]?.name || "Auto"}
                          size="1"
                          src={
                            languageMap[dstLang]?.icon
                              ? `https://flagcdn.com/${languageMap[dstLang].icon}.svg`
                              : undefined
                          }
                        />
                      </IconButton>
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
                            {lang.icon && (
                              <Avatar
                                fallback={lang.name.charAt(0)}
                                alt={lang.name}
                                size="1"
                                src={`https://flagcdn.com/${lang.icon}.svg`}
                              />
                            )}
                            {lang.name}
                          </Flex>
                        </DropdownMenu.Item>
                      ))}
                  </DropdownMenu.Content>
                </DropdownMenu.Root>
              </>
            )}
          </div>

          <div className="flex justify-center gap-1">
            <Tooltip content="Translate">
              <IconButton
                variant="surface"
                color="gray"
                className="shadow-md backdrop-blur-sm"
                loading={loading}
                onClick={doQueryWord}
              >
                <PlayIcon />
              </IconButton>
            </Tooltip>
            <Tooltip content="Save To D1">
              <IconButton
                variant="surface"
                color="gray"
                className="shadow-md backdrop-blur-sm"
                onClick={() => setOpen(true)}
              >
                <DiskIcon />
              </IconButton>
            </Tooltip>
          </div>
        </div>

        <Box mt="2">
          <Text as="label" size="2" weight="bold">
            Text
          </Text>
          <TextArea
            color={query.length === 0 ? "red" : undefined}
            value={query}
            placeholder="Enter text..."
            className="min-h-[40px] resize-y"
            onChange={(e) => setQuery(e.target.value)}
          />
        </Box>

        {selected.startsWith("custom-") && (
          <Flex direction="column" gap="2" mt="2">
            <Flex gap="4">
              <Text as="label" size="2">
                <Flex gap="2" align="center">
                  <Switch
                    checked={googleSearch}
                    onCheckedChange={(e) => setGoogleSearch(e)}
                  />
                  Google Search
                </Flex>
              </Text>

              <Text as="label" size="2">
                <Flex gap="2" align="center">
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
                <Text as="label" size="2" weight="bold">
                  Instruction
                </Text>
                <TextArea
                  value={instruction}
                  className="min-h-[40px] resize-y"
                  onChange={(e) => setInstruction(e.target.value)}
                />
              </Box>
            )}
          </Flex>
        )}

        {result.reasoning && (
          <Box mt="2">
            <Card>
              <Box className="flex-1 max-w-none">
                <Markdown>{result.reasoning}</Markdown>
              </Box>
            </Card>
          </Box>
        )}

        <Box mt="2">
          <Card>
            {result.result ? (
              <Box className="flex-1 max-w-none">
                <Markdown>{result.result}</Markdown>
              </Box>
            ) : (
              <Flex
                className="h-50"
                align="center"
                justify="center"
                direction="column"
              >
                <Text color="gray">
                  Please input translate text and click translate button.
                </Text>
              </Flex>
            )}
          </Card>
        </Box>
      </div>
    </>
  );
}
