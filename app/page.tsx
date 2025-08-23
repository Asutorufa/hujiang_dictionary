"use client"

import { Avatar, Button, Card, CardBody, CardFooter, Select, SelectItem, Textarea } from "@heroui/react";
import { useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";

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
  { key: "ja", name: "Japanese", icon: "jp" },
  { key: "zh", name: "Chinese", icon: "cn" },
  { key: "en", name: "English", icon: "us" },
]

export default function Home() {
  const [selected, setSelected] = useState("ktbk");
  const [query, setQuery] = useState("");
  const [result, setResult] = useState("");
  const [srcLang, setSrcLang] = useState("");
  const [dstLang, setDstLang] = useState("ja");
  const [loading, setLoading] = useState(false);


  return (
    <>
      <div className="p-2">
        <div className="flex w-full justify-center flex-wrap md:flex-nowrap gap-4 items-center">
          <Select label="Translate Type"
            value={selected}
            defaultSelectedKeys={[selected]}
            onChange={(e) => setSelected(e.target.value)}>
            <SelectItem key="weblio">Weblio</SelectItem>
            <SelectItem key="ktbk">コトバンク</SelectItem>
            <SelectItem key="google">Google Translate</SelectItem>
            <SelectItem key="googlev1">Google Translate(old API)</SelectItem>
            <SelectItem key="gpt">GPT OSS 20B</SelectItem>
            <SelectItem key="gemma">Gemma3 27B IT</SelectItem>
            <SelectItem key="llama4">Llama 4 Scout 17B 16E Instruct</SelectItem>
            <SelectItem key="jc">Japanese to Chinese</SelectItem>
            <SelectItem key="cj">Chinese to Japanese</SelectItem>
            <SelectItem key="en">English to Chinese</SelectItem>
          </Select>

          <Select
            isDisabled={!showSelectLang(selected)}
            label="Source Language"
            value={srcLang}
            defaultSelectedKeys={[srcLang]}
            onChange={(e) => setSrcLang(e.target.value)}>
            {[{ key: "", name: "Auto", icon: "un" }, ...languages].map((lang) => (
              <SelectItem key={lang.key}
                startContent={lang.icon ? <Avatar alt={lang.name} className="w-6 h-6" src={`https://flagcdn.com/${lang.icon}.svg`} /> : undefined}>
                {lang.name}
              </SelectItem>
            ))}
          </Select>

          <Select
            isDisabled={!showSelectLang(selected)}
            label="Destination Language"
            defaultSelectedKeys={[dstLang]}
            value={dstLang}
            onChange={(e) => setDstLang(e.target.value)}>
            {languages.map((lang) => (
              <SelectItem key={lang.key} startContent={lang.icon && (
                <Avatar alt={lang.name} className="w-6 h-6" src={`https://flagcdn.com/${lang.icon}.svg`} />
              )}>
                {lang.name}
              </SelectItem>
            ))}
          </Select>
        </div>
        <div className="mt-2">
          <Card>
            <CardBody>
              <Textarea label="Query" type="textarea"
                value={query} onChange={(e) => setQuery(e.target.value)} />
            </CardBody>
            <CardFooter className="flex justify-center">
              <Button color="primary"
                isLoading={loading}
                onPress={() => {
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
                Translate
              </Button>
            </CardFooter>
          </Card>
        </div>

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
      </div>
    </>
  );
}
