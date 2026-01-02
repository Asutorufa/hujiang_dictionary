"use client"

import { Avatar, Button, Card, CardBody, Dropdown, DropdownItem, DropdownMenu, DropdownSection, DropdownTrigger, Switch, Textarea, Tooltip } from "@heroui/react";
import { useCallback, useEffect, useState } from "react";
import rehypeRaw from "rehype-raw";
import remarkGfm from "remark-gfm";
import { Streamdown } from "streamdown";
import { useLocalStorage } from "usehooks-ts";
import { DiskIcon, listModel as listModels, PlayIcon, SaveWordModal } from "./components";

async function queryWord(opts: {
  selected: string,
  query: string,
  instruction: string,
  google_search: boolean,
  srcLang: string,
  dstLang: string,
  custom_llm?: { name: string, model: string }
},
  callback: (data?: { result: string, reasoning?: string }, error?: string) => void) {
  const resp = await fetch("/word/query", {
    method: "POST",
    headers: {
    },
    body: JSON.stringify({
      method: opts.selected,
      word: opts.query,
      instruction: opts.instruction.length > 0 ? opts.instruction : undefined,
      google_search: opts.google_search,
      src_lang: opts.srcLang ? opts.srcLang : undefined,
      dst_lang: opts.dstLang ? opts.dstLang : undefined,
      custom_llm: opts.custom_llm
    }),
  });

  if (resp.ok) {
    callback((await resp.json() as { result: string, reasoning?: string }), undefined);
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
]


const languageMap = Object.fromEntries(
  languages.map(({ key, name, icon }) => [key, { name, icon }])
) as Record<typeof translationSources[number]["key"], { name: string, icon: string }>;


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
  { key: "en", name: "English <-> Chinese", tag: "hujiang" },
];

const translationMap = Object.fromEntries(
  [...translationSources, ...dictSources].map(({ key, name }) => [key, name])
) as Record<typeof translationSources[number]["key"], string>;

export default function Home() {
  const [selected, setSelected] = useLocalStorage("translate_type", "ktbk");
  const [query, setQuery] = useLocalStorage("query", "");
  const [instruction, setInstruction] = useLocalStorage("instruction", "");
  const [googleSearch, setGoogleSearch] = useLocalStorage("google_search", false);
  const [result, setResult] = useLocalStorage<{ result: string, reasoning?: string }>("result_v2", { result: "" });
  const [srcLang, setSrcLang] = useLocalStorage("src_lang", "");
  const [dstLang, setDstLang] = useLocalStorage("dst_lang", "ja");
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);
  const [customModels, setCustomModels] =
    useLocalStorage<{ [key: string]: { name: string, model: string } }[]>("custom_llms_cache", []);

  useEffect(() => {
    listModels((models, error) => {
      if (error) {
        console.log(error);
      } else if (models) {
        const customModels: { [key: string]: { name: string, model: string } }[] = [];
        for (const llm of models) {
          const cm: { [key: string]: { name: string, model: string } } = {};

          for (const model of llm.models) {
            cm[`custom-${llm.name}-${model}`] = { name: llm.name, model: model };
          }

          customModels.push(cm);
        }

        setCustomModels(customModels);
      }
    });
  }, [setCustomModels]);


  const doQueryWord = useCallback(async () => {
    if (!query) return;
    let modelName = selected;
    let customLLM: { name: string, model: string } | undefined;
    if (modelName.startsWith("custom-")) {
      customLLM = customModels.find((cm) => Object.keys(cm).includes(modelName))?.[modelName];
      modelName = "custom_llm";
    }

    setLoading(true);
    await queryWord({
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
          setResult({ result: error })
        } else if (data) {
          setResult(data)
        } else {
          setResult({ result: "NOT FOUND" })
        }
        setLoading(false);
      })
  }, [query, srcLang, dstLang, googleSearch, instruction, selected, customModels, setResult, setLoading]);

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
      <SaveWordModal open={open} onChange={(p) => setOpen(p)} word={query} explain={result.result} type={0} />
      <div className="p-2">

        <div className="sticky flex flex-wrap justify-center top-1 z-50 gap-1">

          <div className="flex gap-1">
            <Dropdown>
              <DropdownTrigger>
                <Button
                  variant="bordered"
                  className="shadow-md backdrop-blur-sm capitalize"
                >
                  <Tooltip content="Translate Method">
                    {translationMap[selected] || customModels.find((cm) => Object.keys(cm).includes(selected))?.[selected].model || "Select"}
                  </Tooltip>
                </Button>
              </DropdownTrigger>
              <DropdownMenu
                selectionMode="single"
                selectedKeys={[selected]}
                onSelectionChange={(e) => e.currentKey && setSelected(e.currentKey)}
              >
                <>
                  <DropdownSection showDivider>
                    {
                      translationSources.map((source) => (
                        <DropdownItem key={source.key}>{source.name}</DropdownItem>
                      ))
                    }
                  </DropdownSection>
                  <DropdownSection showDivider={customModels.length > 0}>
                    {
                      dictSources.map((source) => (
                        <DropdownItem shortcut={source.tag} key={source.key}>{source.name}</DropdownItem>
                      ))
                    }
                  </DropdownSection>
                  {
                    customModels.map((cm, i) => (
                      <DropdownSection key={i} showDivider={i !== customModels.length - 1}>
                        {
                          Object.keys(cm).map((key) => (
                            <DropdownItem shortcut={cm[key].name} key={key}>{cm[key].model}</DropdownItem>
                          ))
                        }
                      </DropdownSection>
                    ))
                  }
                </>
              </DropdownMenu>
            </Dropdown>

            {showSelectLang(selected) &&
              <>
                <Dropdown>
                  <DropdownTrigger>
                    <Button
                      isIconOnly
                      variant="bordered"
                      className="shadow-md backdrop-blur-sm capitalize"
                    >
                      <Tooltip content="Source Language">
                        <Avatar alt={languageMap[srcLang].name} className="w-6 h-6" src={`https://flagcdn.com/${languageMap[srcLang].icon}.svg`} />
                        {/* {languageMap[srcLang].name || "Auto"} */}
                      </Tooltip>
                    </Button>
                  </DropdownTrigger>
                  <DropdownMenu
                    selectionMode="single"
                    selectedKeys={[srcLang]}
                    onSelectionChange={(e) => e.currentKey !== undefined && e.currentKey !== null && setSrcLang(e.currentKey)}
                  >
                    {
                      languages.map((lang) => (
                        <DropdownItem
                          key={lang.key}
                          startContent={lang.icon ? <Avatar alt={lang.name} className="w-6 h-6" src={`https://flagcdn.com/${lang.icon}.svg`} /> : undefined}
                        >
                          {lang.name}
                        </DropdownItem>
                      ))
                    }
                  </DropdownMenu>
                </Dropdown>

                <Dropdown>
                  <DropdownTrigger>
                    <Button
                      isIconOnly
                      variant="bordered"
                      className="shadow-md backdrop-blur-sm capitalize"
                    >
                      <Tooltip content="Target Language">
                        <Avatar alt={languageMap[dstLang].name} className="w-6 h-6" src={`https://flagcdn.com/${languageMap[dstLang].icon}.svg`} />
                        {/* {languageMap[dstLang].name || "Auto"} */}
                      </Tooltip>
                    </Button>
                  </DropdownTrigger>
                  <DropdownMenu
                    selectionMode="single"
                    selectedKeys={[dstLang]}
                    onSelectionChange={(e) => e.currentKey !== undefined && e.currentKey !== null && setDstLang(e.currentKey)}
                  >
                    {
                      languages.filter((lang) => lang.key !== "").map((lang) => (
                        <DropdownItem
                          key={lang.key}
                          startContent={lang.icon ? <Avatar alt={lang.name} className="w-6 h-6" src={`https://flagcdn.com/${lang.icon}.svg`} /> : undefined}
                        >
                          {lang.name}
                        </DropdownItem>
                      ))
                    }
                  </DropdownMenu>
                </Dropdown>
              </>
            }
          </div>

          <div className="flex justify-center gap-1">
            <Tooltip content="Translate">
              <Button
                isIconOnly
                variant="bordered"
                className="shadow-md backdrop-blur-sm"
                isLoading={loading}
                onPress={doQueryWord}
              >
                <PlayIcon />
              </Button>
            </Tooltip>
            <Tooltip content="Save To D1">
              <Button
                isIconOnly
                variant="bordered"
                className="shadow-md backdrop-blur-sm"
                onPress={() => setOpen(true)}
              >
                <DiskIcon />
              </Button>
            </Tooltip>
          </div>
        </div>

        <Textarea
          className="mt-2"
          isInvalid={query.length === 0}
          errorMessage={"Query is empty"}
          label="Text"
          type="textarea"
          value={query}
          height={"full"}
          classNames={{
            input: "resize-y min-h-[40px]",
          }}
          onChange={(e) => setQuery(e.target.value)}
        />


        {(selected.startsWith("custom-")) &&
          <>
            <Switch className="mt-2" isSelected={googleSearch} onValueChange={(e) => setGoogleSearch(e)}>
              Google Search
            </Switch>

            {!googleSearch &&
              <Textarea
                className="mt-2"
                label="Instruction"
                type="textarea"
                value={instruction}
                height={"full"}
                classNames={{
                  input: "resize-y min-h-[40px]",
                }}
                onChange={(e) => setInstruction(e.target.value)}
              />
            }
          </>
        }


        {result.reasoning &&
          <div className="mt-2">
            <Card>
              <CardBody>
                <div className="flex-1 prose max-w-none dark:prose-invert">
                  <Streamdown rehypePlugins={[rehypeRaw]} remarkPlugins={[remarkGfm]}>{result.reasoning}</Streamdown>
                </div>
              </CardBody>
            </Card>
          </div>
        }

        <div className="mt-2">
          <Card>
            <CardBody>
              {result.result ?
                <div className="flex-1 prose max-w-none dark:prose-invert">
                  <Streamdown rehypePlugins={[rehypeRaw]} remarkPlugins={[remarkGfm]}>{result.result}</Streamdown>
                </div>
                :
                <>
                  <div className="h-50 flex items-center justify-center">
                    Please input translate text and click translate button.
                  </div>
                </>
              }
            </CardBody>
          </Card>
        </div>
      </div >
    </>
  );
}
