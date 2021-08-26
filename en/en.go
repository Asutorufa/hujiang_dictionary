package en

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"net/url"
	"regexp"
	"strconv"
	"strings"

	"github.com/PuerkitoBio/goquery"
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
	reEnter, _ := regexp.Compile("\r\n | \n | \r")
	reSpace, _ := regexp.Compile(" +")
	reEnter2Space := func(str string) string {
		return reSpace.ReplaceAllString(reEnter.ReplaceAllString(str, " "), " ")
	}
	reReduceEnter := func(str string) string {
		return strings.TrimSpace(reSpace.ReplaceAllString(strings.Replace(reEnter.ReplaceAllString(str, ""), "\n", "", -1), " "))
	}

	client := http.Client{
		// Transport: &http.Transport{
		// 	DialContext: (&net.Dialer{
		// 		Resolver: &net.Resolver{
		// 			PreferGo: true,
		// 			Dial: func(ctx context.Context, network, address string) (net.Conn, error) {
		// 				return net.DialTimeout(network, "114.114.114.114:53", time.Second*10)
		// 			},
		// 		},
		// 	}).DialContext,
		// },
	}
	var words []*Word
	req, err := http.NewRequest(http.MethodGet, "https://dict.hjenglish.com/w/"+url.PathEscape(str), nil)
	if err != nil {
		log.Println(err)
		return nil
	}
	req.Header.Add("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36")
	req.Header.Add("Cookie", "HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1")
	resp, err := client.Do(req)
	if err != nil {
		log.Println(err)
		return nil
	}
	x, _ := goquery.NewDocumentFromReader(resp.Body)

	wait := make(chan bool, 0)
	for index, s := range x.Find(".word-details-pane").Nodes {
		if index != 0 {
			fmt.Println()
		}
		word := &Word{}
		x := goquery.NewDocumentFromNode(s)

		go func() {
			word.Word = x.Find(".word-text h2").Text()
			if x.Find(".word-info .pronounces .pronounce-value-en").Text() == "" {
				word.Katakana = x.Find(".pronounces span").Text()
				word.AudioEnUrl, _ = x.Find(".pronounces .word-audio").Attr("data-src")
				word.AudioUsUrl = word.AudioEnUrl
			} else {
				enPronounce, _ := x.Find(".word-info .pronounces .word-audio-en").Attr("data-src")
				word.AudioEnUrl = "英 " + x.Find(".word-info .pronounces .pronounce-value-en").Text() + " " + enPronounce
				usPronounce, _ := x.Find(".word-info .pronounces .word-audio").Last().Attr("data-src")
				word.AudioUsUrl = "美 " + x.Find(".word-info .pronounces .pronounce-value-us").Text() + " " + usPronounce
			}

			if x.Find(".simple p .simple-definition a").Text() == "" {
				for _, s := range x.Find(".simple p").Nodes {
					x := goquery.NewDocumentFromNode(s)
					word.Simple = append(word.Simple, reReduceEnter(x.Text()))
				}
			} else {
				for _, s := range x.Find(".simple p .simple-definition a").Nodes {
					word.Simple = append(word.Simple, reReduceEnter(goquery.NewDocumentFromNode(s).Text()))
				}
			}
			wait <- true
		}()

		go func() {
			word.Detail = []*Detail{}
			for _, s := range x.Find(".word-details-pane-content .word-details-item").Nodes {
				x := goquery.NewDocumentFromNode(s)
				for _, s := range x.Find(".word-details-item-content .detail-groups dl").Nodes {
					x := goquery.NewDocumentFromNode(s)
					detail := &Detail{}
					detail.Attribute = reReduceEnter(x.Find("dt").Text())

					for _, s := range x.Find("dd").Nodes {
						x := goquery.NewDocumentFromNode(s)
						if detail.ExplainsAndExample == nil {
							detail.ExplainsAndExample = []*ExplainsAndExample{}
						}
						explain := strings.TrimSpace(reReduceEnter(x.Find("h3").Text()))

						explainsAndExampleTmp := &ExplainsAndExample{}
						explainsAndExampleTmp.Explain = explain
						for _, s := range x.Find("ul li").Nodes {
							x := goquery.NewDocumentFromNode(s)
							eg := reReduceEnter(x.Find(".def-sentence-from").Text())
							eg2 := reReduceEnter(x.Find(".def-sentence-to").Text())
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
			for _, s := range x.Find(".word-details-item-content .phrase-items li").Nodes {
				x := goquery.NewDocumentFromNode(s)
				word.Phrase = append(word.Phrase, reReduceEnter(x.Text()))
			}
			wait <- true
		}()

		go func() {
			word.EnglishExplains = []*EnglishExplain{}
			for _, s := range x.Find(".word-details-item-content .enen-groups dl").Nodes {
				x := goquery.NewDocumentFromNode(s)
				englishExplainTmp := &EnglishExplain{}
				englishExplainTmp.Attribute = reReduceEnter(x.Find("dt").Text())
				for _, s := range x.Find("dd").Nodes {
					x := goquery.NewDocumentFromNode(s)
					englishExplainTmp.Explains = append(englishExplainTmp.Explains, reEnter2Space(x.Text()))
				}
				word.EnglishExplains = append(word.EnglishExplains, englishExplainTmp)
			}
			wait <- true
		}()

		go func() {
			for _, s := range x.Find(".word-details-item-content .inflections-items li").Nodes {
				x := goquery.NewDocumentFromNode(s)
				word.Inflections = append(word.Inflections, reReduceEnter(x.Text()))
			}
			wait <- true
		}()

		go func() {
			for _, s := range x.Find(".word-details-item-content .syn table tbody tr td a").Nodes {
				word.Synonym = append(word.Synonym, reReduceEnter(goquery.NewDocumentFromNode(s).Text()))
			}
			wait <- true
		}()

		go func() {
			for _, s := range x.Find(".word-details-item-content .ant table tbody tr td a").Nodes {
				word.Antonym = append(word.Antonym, reReduceEnter(goquery.NewDocumentFromNode(s).Text()))
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
