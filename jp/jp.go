package jp

import (
	"encoding/json"
	"fmt"
	"github.com/opesun/goquery"
	"io/ioutil"
	"log"
	"net/http"
	"net/url"
	"regexp"
	"strconv"
	"strings"
)

type SimpleExplain struct {
	Attribute string   `json:"attribute"`
	Explains  []string `json:"explains"`
}

type ExplainsAndExample struct {
	Explain string     `json:"explain"`
	Example [][]string `json:"example"`
}
type Detail struct {
	Attribute          string                `json:"attribute"`
	ExplainsAndExample []*ExplainsAndExample `json:"explains_and_example"`
}

type Word struct {
	Word     string           `json:"word"`
	Katakana string           `json:"katakana"`
	AudioUrl string           `json:"audio_url"`
	Simple   []*SimpleExplain `json:"simple"`
	Detail   []*Detail        `json:"detail"`
}

func Get(str string) []*Word {
	var words []*Word
	req, err := http.NewRequest(http.MethodGet, "https://dict.hjenglish.com/jp/jc/"+url.QueryEscape(str), nil)
	if err != nil {
		log.Println(err)
	}
	req.Header.Add("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36")
	req.Header.Add("Cookie", "HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1")
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		log.Println(err)
	}
	s, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		log.Println(err)
	}
	x, _ := goquery.ParseString(string(s))

	for _, s := range x.Find(".word-details-pane").HtmlAll() {
		word := &Word{}
		x, _ := goquery.ParseString(s)
		word.Word, word.Katakana, word.AudioUrl = x.Find(".word-text h2").Text(), x.Find(".pronounces span").Text(), x.Find(".pronounces .word-audio").Attr("data-src")
		re, _ := regexp.Compile("\r\n | \n | \r")
		re2, _ := regexp.Compile(" +")
		re3, _ := regexp.Compile("\n。")

		word.Simple = []*SimpleExplain{}
		for _, s := range x.Find(".simple").HtmlAll() {
			x, _ := goquery.ParseString(s)
			list := x.Find("ul").HtmlAll()
			for index, s := range x.Find("h2").HtmlAll() {
				simpleTmp := &SimpleExplain{}
				simpleTmp.Attribute = s
				x, _ := goquery.ParseString(list[index])
				for _, s := range x.Find("li").HtmlAll() {
					x, _ := goquery.ParseString(s)
					simpleTmp.Explains = append(simpleTmp.Explains, x.Text())
				}
				word.Simple = append(word.Simple, simpleTmp)
			}
		}

		word.Detail = []*Detail{}
		for _, s := range x.Find(".word-details-pane-content .word-details-item").HtmlAll() {
			x, _ := goquery.ParseString(s)
			reAll2 := func(str string) string {
				return re3.ReplaceAllString(re2.ReplaceAllString(re.ReplaceAllString(str, ""), ""), "。")
			}
			for _, s := range x.Find(".word-details-item-content .detail-groups dl").HtmlAll() {
				x, _ := goquery.ParseString(s)
				detailTmp := &Detail{}
				detailTmp.Attribute = strings.Replace(reAll2(x.Find("dt").Text()), "\n", "", -1)
				for _, s := range x.Find("dd").HtmlAll() {
					x, _ := goquery.ParseString(s)
					explainsAndExampleTmp := &ExplainsAndExample{}
					explainsAndExampleTmp.Explain = strings.Replace(reAll2(x.Find("h3").Text()), "\n", "", -1)
					for _, s := range x.Find("ul li").HtmlAll() {
						x, _ := goquery.ParseString(s)
						from := strings.Replace(reAll2(x.Find(".def-sentence-from").Text()), "\n", "", -1)
						to := strings.Replace(reAll2(x.Find(".def-sentence-to").Text()), "\n", "", -1)
						tmp := []string{from, to}
						explainsAndExampleTmp.Example = append(explainsAndExampleTmp.Example, tmp)
					}
					detailTmp.ExplainsAndExample = append(detailTmp.ExplainsAndExample, explainsAndExampleTmp)
				}
				word.Detail = append(word.Detail, detailTmp)
			}
		}
		words = append(words, word)
	}
	return words
}

func GetJson(str string) (string, error) {
	s, err := json.MarshalIndent(Get(str), "", " ")
	return string(s), err
}

func Show(str string) {
	x := Get(str)
	for _, s := range x {
		fmt.Println(s.Word, s.Katakana, s.AudioUrl)

		fmt.Println("simple explain:")
		for index := range s.Simple {
			fmt.Println(" " + s.Simple[index].Attribute)
			for _, s := range s.Simple[index].Explains {
				fmt.Println("   " + s)
			}
		}

		fmt.Println("More Detail:")
		for index := range s.Detail {
			fmt.Println(" word attribute: " + s.Detail[index].Attribute)
			tmp := s.Detail[index].ExplainsAndExample
			for index := range tmp {
				fmt.Println("  " + strconv.Itoa(index+1) + "." + tmp[index].Explain)
				exampleTmp := tmp[index].Example
				for index := range exampleTmp {
					for i := range exampleTmp[index] {
						switch i {
						case 0:
							fmt.Println("    " + strconv.Itoa(index+1) + ")" + exampleTmp[index][i])
						case 1:
							fmt.Println("      " + exampleTmp[index][i])

						}
					}
				}
			}
		}
	}
}
