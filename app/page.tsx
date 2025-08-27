"use client"

import { Avatar, Button, Card, CardBody, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Textarea } from "@heroui/react";
import { useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { useLocalStorage } from "usehooks-ts";
import { DiskIcon, PlayIcon, SaveWordModal } from "./components";

type QueryWordResponse = {
  result: string;
}

async function queryWord(selected: string, query: string, srcLang: string, dstLang: string,
  callback: (data?: string, error?: string) => void) {
  fetch("/word/query", {
    method: "POST",
    headers: {
    },
    body: JSON.stringify({
      method: selected,
      word: query,
      src_lang: srcLang ? srcLang : undefined,
      dst_lang: dstLang ? dstLang : undefined,
    }),
  })
    .then((res) => res.json() as Promise<QueryWordResponse>)
    .then((data) => {
      callback(data.result, undefined);
    })
    .catch((error) => {
      callback(undefined, error.message);
    });
}

function showSelectLang(selected: string) {
  switch (selected) {
    case "google":
    case "googlev1":
    case "gpt":
    case "gemma":
    case "llama4":
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
  { key: "weblio", name: "Weblio" },
  { key: "ktbk", name: "コトバンク" },
  { key: "google", name: "Google Translate" },
  { key: "googlev1", name: "Google Translate(old API)" },
  { key: "gpt", name: "GPT OSS 20B" },
  { key: "gemma", name: "Gemma3 27B IT" },
  { key: "llama4", name: "Llama 4 Scout 17B 16E Instruct" },
  { key: "jc", name: "Japanese to Chinese" },
  { key: "cj", name: "Chinese to Japanese" },
  { key: "en", name: "English to Chinese" },
];

const translationMap = Object.fromEntries(
  translationSources.map(({ key, name }) => [key, name])
) as Record<typeof translationSources[number]["key"], string>;


export default function Home() {
  const [selected, setSelected] = useLocalStorage("translate_type", "ktbk");
  const [query, setQuery] = useLocalStorage("query", "");
  const [result, setResult] = useLocalStorage("result", "");
  const [srcLang, setSrcLang] = useLocalStorage("src_lang", "");
  const [dstLang, setDstLang] = useLocalStorage("dst_lang", "ja");
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);

  return (
    <>
      <SaveWordModal open={open} onChange={(p) => setOpen(p)} word={query} explain={result} type={0} />
      <div className="p-2">

        <div className="sticky flex flex-wrap justify-center top-1 z-50 gap-1">

          <div className="flex gap-1">
            <Dropdown>
              <DropdownTrigger>
                <Button
                  variant="bordered"
                  className="shadow-md backdrop-blur-sm capitalize"
                >
                  {translationMap[selected] || "Select"}
                </Button>
              </DropdownTrigger>
              <DropdownMenu
                selectionMode="single"
                selectedKeys={[selected]}
                onSelectionChange={(e) => e.currentKey && setSelected(e.currentKey)}
              >
                {
                  translationSources.map((source) => (
                    <DropdownItem key={source.key}>{source.name}</DropdownItem>
                  ))
                }
              </DropdownMenu>
            </Dropdown>

            <Dropdown>
              <DropdownTrigger>
                <Button
                  isIconOnly
                  isDisabled={!showSelectLang(selected)}
                  variant="bordered"
                  className="shadow-md backdrop-blur-sm capitalize"
                >
                  <Avatar alt={languageMap[srcLang].name} className="w-6 h-6" src={`https://flagcdn.com/${languageMap[srcLang].icon}.svg`} />
                  {/* {languageMap[srcLang].name || "Auto"} */}
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
                  isDisabled={!showSelectLang(selected)}
                  variant="bordered"
                  className="shadow-md backdrop-blur-sm capitalize"
                >
                  <Avatar alt={languageMap[dstLang].name} className="w-6 h-6" src={`https://flagcdn.com/${languageMap[dstLang].icon}.svg`} />
                  {/* {languageMap[dstLang].name || "Auto"} */}
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
          </div>

          <div className="flex justify-center gap-1">
            <Button
              isIconOnly
              variant="bordered"
              className="shadow-md backdrop-blur-sm"
              isLoading={loading}
              onPress={() => {
                if (!query) return;
                setLoading(true);
                queryWord(selected, query, srcLang, dstLang, (data, error) => {
                  console.log(data);
                  if (error) {
                    setResult(error)
                  } else if (data) {
                    setResult(data)
                  } else {
                    setResult("NOT FOUND")
                  }
                  setLoading(false);
                })
              }}
            >
              <PlayIcon />
            </Button>
            <Button
              isIconOnly
              variant="bordered"
              className="shadow-md backdrop-blur-sm"
              onPress={() => setOpen(true)}
            >
              <DiskIcon />
            </Button>
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

        <div className="mt-2">
          <Card>
            <CardBody>

              {result ?
                <div className="flex-1">
                  <Markdown remarkPlugins={[remarkGfm]}>{result}</Markdown>
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
