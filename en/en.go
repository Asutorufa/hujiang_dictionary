package en

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

// Detail word detail explains
// Attribute word attribute
// ExplainsAndExample {explains:[[example],[example]]}  | example: [eg,eg2]
type ExplainsAndExample struct {
	Explain string     `json:"explain"`
	Example [][]string `json:"example"`
}
type Detail struct {
	Attribute          string                `json:"attribute"`
	ExplainsAndExample []*ExplainsAndExample `json:"explains_and_example"`
}

type EnglishExplain struct {
	Attribute string   `json:"attribute"`
	Explains  []string `json:"explains"`
}

type Word struct {
	Word            string            `json:"word"`
	Katakana        string            `json:"katakana"`
	Roma            string            `json:"roma"`
	AudioEnUrl      string            `json:"audio_en_url"`
	AudioUsUrl      string            `json:"audio_us_url"`
	EnglishExplains []*EnglishExplain `json:"english_explains"`
	Phrase          []string          `json:"phrase"`
	Synonym         []string          `json:"synonym"`
	Antonym         []string          `json:"antonym"`
	Inflections     []string          `json:"inflections"`
	Simple          []string          `json:"simple"`
	Detail          []*Detail         `json:"detail"`
}

func Get(str string) []*Word {
	re, _ := regexp.Compile("\r\n | \n | \r")
	re2, _ := regexp.Compile(" +")
	reAll := func(str string) string {
		return re2.ReplaceAllString(re.ReplaceAllString(str, " "), " ")
	}
	reAll2 := func(str string) string {
		return strings.TrimSpace(re2.ReplaceAllString(strings.Replace(re.ReplaceAllString(str, ""), "\n", "", -1), " "))
	}

	var words []*Word
	req, err := http.NewRequest(http.MethodGet, "https://dict.hjenglish.com/w/"+url.PathEscape(str), nil)
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

	wait := make(chan bool, 0)
	for index, s := range x.Find(".word-details-pane").HtmlAll() {
		if index != 0 {
			fmt.Println()
		}
		word := &Word{}
		x, _ := goquery.ParseString(s)

		go func() {
			word.Word = x.Find(".word-text h2").Text()
			if x.Find(".word-info .pronounces .pronounce-value-en").Text() == "" {
				word.Katakana, word.AudioEnUrl = x.Find(".pronounces span").Text(), x.Find(".pronounces .word-audio").Attr("data-src")
				word.AudioUsUrl = word.AudioEnUrl
			} else {
				word.AudioEnUrl = "英 " + x.Find(".word-info .pronounces .pronounce-value-en").Text() + " " + x.Find(".word-info .pronounces .word-audio-en").Attr("data-src")
				word.AudioUsUrl = "美 " + x.Find(".word-info .pronounces .pronounce-value-us").Text() + " " + x.Find(".word-info .pronounces .word-audio").Last().Attr("data-src")
			}

			if x.Find(".simple p .simple-definition a").Text() == "" {
				for _, s := range x.Find(".simple p").HtmlAll() {
					x, _ := goquery.ParseString(s)
					word.Simple = append(word.Simple, reAll2(x.Text()))
				}
			} else {
				for _, s := range x.Find(".simple p .simple-definition a").HtmlAll() {
					x, _ := goquery.ParseString(s)
					word.Simple = append(word.Simple, reAll2(x.Text()))
				}
			}
			wait <- true
		}()

		go func() {
			word.Detail = []*Detail{}
			for _, s := range x.Find(".word-details-pane-content .word-details-item").HtmlAll() {
				x, _ := goquery.ParseString(s)
				for _, s := range x.Find(".word-details-item-content .detail-groups dl").HtmlAll() {
					x, _ := goquery.ParseString(s)
					detail := &Detail{}
					detail.Attribute = reAll2(x.Find("dt").Text())

					for _, s := range x.Find("dd").HtmlAll() {
						x, _ := goquery.ParseString(s)
						if detail.ExplainsAndExample == nil {
							detail.ExplainsAndExample = []*ExplainsAndExample{}
						}
						explain := strings.TrimSpace(reAll2(x.Find("h3").Text()))

						explainsAndExampleTmp := &ExplainsAndExample{}
						explainsAndExampleTmp.Explain = explain
						for _, s := range x.Find("ul li").HtmlAll() {
							x, _ := goquery.ParseString(s)
							eg := reAll2(x.Find(".def-sentence-from").Text())
							eg2 := reAll2(x.Find(".def-sentence-to").Text())
							explainsAndExampleTmp.Example = append(explainsAndExampleTmp.Example, []string{eg, eg2})
						}
						detail.ExplainsAndExample = append(detail.ExplainsAndExample, explainsAndExampleTmp)
					}
					word.Detail = append(word.Detail, detail)
				}
			}
			wait <- true
		}()

		go func() {
			for _, s := range x.Find(".word-details-item-content .phrase-items li").HtmlAll() {
				x, _ := goquery.ParseString(s)
				word.Phrase = append(word.Phrase, reAll2(x.Text()))
			}
			wait <- true
		}()

		go func() {
			word.EnglishExplains = []*EnglishExplain{}
			for _, s := range x.Find(".word-details-item-content .enen-groups dl").HtmlAll() {
				x, _ := goquery.ParseString(s)
				englishExplainTmp := &EnglishExplain{}
				englishExplainTmp.Attribute = reAll2(x.Find("dt").Text())
				for _, s := range x.Find("dd").HtmlAll() {
					x, _ := goquery.ParseString(s)
					englishExplainTmp.Explains = append(englishExplainTmp.Explains, reAll(x.Text()))
				}
				word.EnglishExplains = append(word.EnglishExplains, englishExplainTmp)
			}
			wait <- true
		}()

		go func() {
			for _, s := range x.Find(".word-details-item-content .inflections-items li").HtmlAll() {
				x, _ := goquery.ParseString(s)
				word.Inflections = append(word.Inflections, reAll2(x.Text()))
			}
			wait <- true
		}()

		go func() {
			for _, s := range x.Find(".word-details-item-content .syn table tbody tr td a").HtmlAll() {
				word.Synonym = append(word.Synonym, reAll2(s))
			}
			wait <- true
		}()

		go func() {
			for _, s := range x.Find(".word-details-item-content .ant table tbody tr td a").HtmlAll() {
				word.Antonym = append(word.Antonym, reAll2(s))
			}
			wait <- true
		}()

		<-wait
		<-wait
		<-wait
		<-wait
		<-wait
		<-wait
		<-wait
		words = append(words, word)
	}
	close(wait)
	return words
}

func GetJson(str string) (s string, err error) {
	x, err := json.MarshalIndent(Get(str), "", "  ")
	return string(x), err
}

func Show(str string) {
	x := Get(str)
	for _, s := range x {
		fmt.Println(s.Word)
		fmt.Println(s.Katakana, s.Roma)
		fmt.Println(s.AudioUsUrl)
		fmt.Println(s.AudioEnUrl)

		fmt.Println("simple explain:")
		for index := range s.Simple {
			fmt.Println(" " + s.Simple[index])
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

		fmt.Println("English Explains:")
		for index := range s.EnglishExplains {
			fmt.Println(" " + s.EnglishExplains[index].Attribute)
			for i := range s.EnglishExplains[index].Explains {
				fmt.Println("   " + strconv.Itoa(i+1) + "." + s.EnglishExplains[index].Explains[i])
			}
		}

		fmt.Println("inflections:")
		for index, x := range s.Inflections {
			fmt.Println(" " + strconv.Itoa(index+1) + "." + x)
		}

		fmt.Println("phrase:")
		for index, x := range s.Phrase {
			fmt.Println(" " + strconv.Itoa(index+1) + "." + x)
		}

		fmt.Println("synonym:")
		for index := range s.Synonym {
			fmt.Println(" " + strconv.Itoa(index+1) + "." + s.Synonym[index])
		}
		fmt.Println("antonym:")
		for index := range s.Antonym {
			fmt.Println(" " + strconv.Itoa(index+1) + "." + s.Antonym[index])
		}
	}
}
