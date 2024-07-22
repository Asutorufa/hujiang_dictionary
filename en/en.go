package en

import (
	"encoding/json"
	"log"
	"net/http"
	"net/url"
	"regexp"
	"strconv"
	"strings"

	"github.com/Asutorufa/hujiang_dictionary/httpclient"
	"github.com/Asutorufa/hujiang_dictionary/utils"
	"github.com/PuerkitoBio/goquery"
)

// Detail word detail explains
// Attribute word attribute
// ExplainsAndExample {explains:[[example],[example]]}  | example: [eg,eg2]
type ExplainsAndExample struct {
	Explain string      `json:"explain"`
	Example [][2]string `json:"example"`
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

	client := httpclient.DefaultClient

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
	utils.Each(x.Find(".word-details-pane"), func(i int, x *goquery.Document) {
		word := Word{}

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
		utils.Each(simpleP, func(i int, s *goquery.Document) {
			word.Simple = append(word.Simple, reReduceEnter(s.Text()))
		})

		utils.Each(simpleP.Find(".simple-definition a"), func(i int, s *goquery.Document) {
			word.Simple = append(word.Simple, reReduceEnter(s.Text()))
		})

		wordDetailsItemContent := x.Find(".word-details-item-content")

		utils.Each(wordDetailsItemContent.Find(".detail-groups dl"), func(i int, s *goquery.Document) {
			detail := Detail{Attribute: reReduceEnter(s.Find("dt").Text())}

			utils.Each(s.Find("dd"), func(i int, x *goquery.Document) {
				explainsAndExampleTmp := ExplainsAndExample{Explain: strings.TrimSpace(reReduceEnter(x.Find("h3").Text()))}
				utils.Each(x.Find("ul li"), func(i int, s *goquery.Document) {
					explainsAndExampleTmp.Example = append(
						explainsAndExampleTmp.Example,
						[2]string{
							reReduceEnter(s.Find(".def-sentence-from").Text()),
							reReduceEnter(s.Find(".def-sentence-to").Text()),
						},
					)
				})
				detail.ExplainsAndExample = append(detail.ExplainsAndExample, explainsAndExampleTmp)
			})
			word.Detail = append(word.Detail, detail)
		})

		utils.Each(wordDetailsItemContent.Find(".phrase-items li"), func(i int, s *goquery.Document) {
			word.Phrase = append(word.Phrase, reReduceEnter(s.Text()))
		})

		utils.Each(wordDetailsItemContent.Find(".enen-groups dl"), func(i int, s *goquery.Document) {
			englishExplainTmp := EnglishExplain{Attribute: reReduceEnter(s.Find("dt").Text())}

			utils.Each(s.Find("dd"), func(i int, s *goquery.Document) {
				englishExplainTmp.Explains = append(englishExplainTmp.Explains, reEnter2Space(s.Text()))
			})
			word.EnglishExplains = append(word.EnglishExplains, englishExplainTmp)
		})

		utils.Each(wordDetailsItemContent.Find(".inflections-items li"), func(i int, s *goquery.Document) {
			word.Inflections = append(word.Inflections, reReduceEnter(s.Text()))
		})

		utils.Each(wordDetailsItemContent.Find(".syn table tbody tr td a"), func(i int, s *goquery.Document) {
			word.Synonym = append(word.Synonym, reReduceEnter(s.Text()))
		})

		utils.Each(wordDetailsItemContent.Find(".ant table tbody tr td a"), func(i int, s *goquery.Document) {
			word.Antonym = append(word.Antonym, reReduceEnter(s.Text()))
		})

		words = append(words, word)
	})
	return words
}

func GetJson(str string) (s string, err error) {
	x, err := json.MarshalIndent(Get(str), "", "  ")
	return string(x), err
}

func FormatString(str string) string {
	return convertToString(Get(str))
}

func convertToString(w []Word) string {
	str := strings.Builder{}
	for _, s := range w {
		str.WriteString(s.Word)
		str.WriteByte('\n')
		str.WriteString(s.Katakana)
		str.WriteByte(' ')
		str.WriteString(s.Roma)
		str.WriteByte('\n')
		str.WriteString(s.AudioUsUrl)
		str.WriteByte('\n')
		str.WriteString(s.AudioEnUrl)
		str.WriteByte('\n')

		for i := range s.Simple {
			if i == 0 {
				str.WriteString("simple explain:\n")
			}
			str.WriteByte(' ')
			str.WriteString(s.Simple[i])
			str.WriteByte('\n')
		}

		for i := range s.Detail {
			if i == 0 {
				str.WriteString("More Detail:\n")
			}

			str.WriteString(" word attribute: ")
			str.WriteString(s.Detail[i].Attribute)
			str.WriteByte('\n')

			for i2 := range s.Detail[i].ExplainsAndExample {
				str.WriteString("  ")
				str.WriteString(strconv.Itoa(i2 + 1))
				str.WriteByte('.')
				str.WriteString(s.Detail[i].ExplainsAndExample[i2].Explain)
				str.WriteByte('\n')

				for i3 := range s.Detail[i].ExplainsAndExample[i2].Example {
					str.WriteString("    ")
					str.WriteString(strconv.Itoa(i3 + 1))
					str.WriteByte(')')
					str.WriteString(s.Detail[i].ExplainsAndExample[i2].Example[i3][0])
					str.WriteByte('\n')

					str.WriteString("      ")
					str.WriteString(s.Detail[i].ExplainsAndExample[i2].Example[i3][1])
					str.WriteByte('\n')
				}
			}
		}

		for i := range s.EnglishExplains {
			if i == 0 {
				str.WriteString("English Explains:\n")
			}
			str.WriteByte(' ')
			str.WriteString(s.EnglishExplains[i].Attribute)
			str.WriteByte('\n')

			for i2 := range s.EnglishExplains[i].Explains {
				str.WriteString("   ")
				str.WriteString(strconv.Itoa(i2 + 1))
				str.WriteByte('.')
				str.WriteString(s.EnglishExplains[i].Explains[i2])
				str.WriteByte('\n')
			}
		}

		for i := range s.Inflections {
			if i == 0 {
				str.WriteString("inflections:\n")
			}
			str.WriteByte(' ')
			str.WriteString(strconv.Itoa(i + 1))
			str.WriteByte('.')
			str.WriteString(s.Inflections[i])
			str.WriteByte('\n')
		}

		for i := range s.Phrase {
			if i == 0 {
				str.WriteString("phrase:\n")
			}
			str.WriteByte(' ')
			str.WriteString(strconv.Itoa(i + 1))
			str.WriteByte('.')
			str.WriteString(s.Phrase[i])
			str.WriteByte('\n')
		}

		for i := range s.Synonym {
			if i == 0 {
				str.WriteString("synonym:\n")
			}
			str.WriteByte(' ')
			str.WriteString(strconv.Itoa(i + 1))
			str.WriteByte('.')
			str.WriteString(s.Synonym[i])
			str.WriteByte('\n')
		}

		for i := range s.Antonym {
			if i == 0 {
				str.WriteString("antonym:\n")
			}
			str.WriteByte(' ')
			str.WriteString(strconv.Itoa(i + 1))
			str.WriteByte('.')
			str.WriteString(s.Antonym[i])
			str.WriteByte('\n')
		}
	}

	return str.String()
}
