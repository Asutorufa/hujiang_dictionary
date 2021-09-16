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
	"sync"

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
	Attribute          string               `json:"attribute"`
	ExplainsAndExample []ExplainsAndExample `json:"explains_and_example"`
}

type EnglishExplain struct {
	Attribute string   `json:"attribute"`
	Explains  []string `json:"explains"`
}

type Word struct {
	Word            string           `json:"word"`
	Katakana        string           `json:"katakana"`
	Roma            string           `json:"roma"`
	AudioEnUrl      string           `json:"audio_en_url"`
	AudioUsUrl      string           `json:"audio_us_url"`
	EnglishExplains []EnglishExplain `json:"english_explains"`
	Phrase          []string         `json:"phrase"`
	Synonym         []string         `json:"synonym"`
	Antonym         []string         `json:"antonym"`
	Inflections     []string         `json:"inflections"`
	Simple          []string         `json:"simple"`
	Detail          []Detail         `json:"detail"`
}

var (
	reSpace       = regexp.MustCompile(" +")
	reEnter       = regexp.MustCompile("\n+")
	reReduceEnter = func(str string) string {
		return strings.TrimSpace(reSpace.ReplaceAllString(strings.Replace(reEnter.ReplaceAllString(str, ""), "\n", "", -1), " "))
	}
	reEnter2Space = func(str string) string {
		return reSpace.ReplaceAllString(reEnter.ReplaceAllString(str, " "), " ")
	}
	userAgent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36"
	cookie    = "HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1"
)

func Get(str string) []Word {

	client := http.Client{}
	req, err := http.NewRequest(http.MethodGet, "https://dict.hjenglish.com/w/"+url.PathEscape(str), nil)
	if err != nil {
		log.Println(err)
		return nil
	}
	req.Header.Add("User-Agent", userAgent)
	req.Header.Add("Cookie", cookie)
	resp, err := client.Do(req)
	if err != nil {
		log.Println(err)
		return nil
	}

	// d, _ := ioutil.ReadAll(resp.Body)
	// fmt.Println(string(d))

	x, err := goquery.NewDocumentFromReader(resp.Body)
	if err != nil {
		log.Println(err)
		return nil
	}

	var words []Word
	w := sync.WaitGroup{}
	x.Find(".word-details-pane").Each(func(i int, x *goquery.Selection) {
		word := Word{}

		w.Add(1)
		go func() {
			defer w.Done()
			word.Word = x.Find(".word-text h2").Text()
			pronounces := x.Find(".pronounces")
			en := pronounces.Find(".pronounce-value-en")
			if en.Size() == 0 {
				word.Katakana = pronounces.Find("span").Text()
				word.AudioEnUrl = pronounces.Find(".word-audio").AttrOr("data-src", "")
				word.AudioUsUrl = word.AudioEnUrl
			} else {
				word.AudioEnUrl = en.Text() + " " + pronounces.Find(".word-audio-en").AttrOr("data-src", "")
				word.AudioUsUrl = pronounces.Find(".pronounce-value-us").Text() + " " + pronounces.Find(".word-audio").Last().AttrOr("data-src", "")
			}

			simpleP := x.Find(".simple p")
			simpleP.Each(func(i int, s *goquery.Selection) {
				word.Simple = append(word.Simple, reReduceEnter(s.Text()))
			})

			simpleP.Find(".simple-definition a").Each(func(i int, s *goquery.Selection) {
				word.Simple = append(word.Simple, reReduceEnter(s.Text()))
			})
		}()

		wordDetailsItemContent := x.Find(".word-details-item-content")

		w.Add(1)
		go func() {
			defer w.Done()
			wordDetailsItemContent.Find(".detail-groups dl").Each(func(i int, s *goquery.Selection) {
				detail := Detail{Attribute: reReduceEnter(s.Find("dt").Text())}

				s.Find("dd").Each(func(i int, x *goquery.Selection) {
					explainsAndExampleTmp := ExplainsAndExample{Explain: strings.TrimSpace(reReduceEnter(x.Find("h3").Text()))}
					x.Find("ul li").Each(func(i int, s *goquery.Selection) {
						explainsAndExampleTmp.Example = append(
							explainsAndExampleTmp.Example,
							[]string{
								reReduceEnter(s.Find(".def-sentence-from").Text()),
								reReduceEnter(s.Find(".def-sentence-to").Text()),
							},
						)
					})
					detail.ExplainsAndExample = append(detail.ExplainsAndExample, explainsAndExampleTmp)
				})
				word.Detail = append(word.Detail, detail)
			})
		}()

		w.Add(1)
		go func() {
			defer w.Done()
			wordDetailsItemContent.Find(".phrase-items li").Each(func(i int, s *goquery.Selection) {
				word.Phrase = append(word.Phrase, reReduceEnter(s.Text()))
			})
		}()

		w.Add(1)
		go func() {
			defer w.Done()
			wordDetailsItemContent.Find(".enen-groups dl").Each(func(i int, s *goquery.Selection) {
				englishExplainTmp := EnglishExplain{Attribute: reReduceEnter(s.Find("dt").Text())}

				s.Find("dd").Each(func(i int, s *goquery.Selection) {
					englishExplainTmp.Explains = append(englishExplainTmp.Explains, reEnter2Space(s.Text()))
				})
				word.EnglishExplains = append(word.EnglishExplains, englishExplainTmp)
			})
		}()

		w.Add(1)
		go func() {
			defer w.Done()
			wordDetailsItemContent.Find(".inflections-items li").Each(func(i int, s *goquery.Selection) {
				word.Inflections = append(word.Inflections, reReduceEnter(s.Text()))
			})
		}()

		w.Add(1)
		go func() {
			defer w.Done()
			wordDetailsItemContent.Find(".syn table tbody tr td a").Each(func(i int, s *goquery.Selection) {
				word.Synonym = append(word.Synonym, reReduceEnter(s.Text()))
			})
		}()

		w.Add(1)
		go func() {
			defer w.Done()
			wordDetailsItemContent.Find(".ant table tbody tr td a").Each(func(i int, s *goquery.Selection) {
				word.Antonym = append(word.Antonym, reReduceEnter(s.Text()))
			})
		}()

		w.Wait()
		words = append(words, word)
	})
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
