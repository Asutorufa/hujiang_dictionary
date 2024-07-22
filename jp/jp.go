package jp

import (
	"encoding/json"
	"fmt"
	"io"
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

type SimpleExplain struct {
	Attribute string   `json:"attribute"`
	Explains  []string `json:"explains"`
}

type ExplainsAndExample struct {
	Explain string      `json:"explain"`
	Example [][2]string `json:"example"`
}
type Detail struct {
	Source             string               `json:"source"`
	Attribute          string               `json:"attribute"`
	ExplainsAndExample []ExplainsAndExample `json:"explains_and_example"`
}

type Word struct {
	Word     string          `json:"word"`
	Katakana string          `json:"katakana"`
	AudioUrl string          `json:"audio_url"`
	Simple   []SimpleExplain `json:"simple"`
	Detail   []Detail        `json:"detail"`
}

var (
	reEnterSpace = regexp.MustCompile("\r\n | \n | \r")
	reSpace      = regexp.MustCompile(" +")
	reEnterDot   = regexp.MustCompile("\n。")
	reEnter      = regexp.MustCompile("\n+")
	reSum        = func(str string) string {
		return strings.TrimSpace(reEnterDot.ReplaceAllString(reSpace.ReplaceAllString(reEnterSpace.ReplaceAllString(reEnter.ReplaceAllString(str, "\n"), ""), ""), "。"))
	}
	userAgent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36"
	cookie    = "HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1"
)

func Get(str string) []Word {
	return get(str, "jc")
}

func get(str, u string) []Word {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("https://dict.hjenglish.com/jp/%s/%s", u, url.PathEscape(str)), nil)
	if err != nil {
		log.Println(err)
		return nil
	}
	req.Header.Add("User-Agent", userAgent)
	req.Header.Add("Cookie", cookie)

	resp, err := httpclient.DefaultClient.Do(req)
	if err != nil {
		log.Println(err)
		return nil
	}

	return getWords(resp.Body)
}

func GetCN(str string) []Word {
	return get(str, "cj")
}

func getWords(r io.Reader) []Word {
	var words []Word
	x, err := goquery.NewDocumentFromReader(r)
	if err != nil {
		panic(err)
	}

	utils.Each(x.Find(".word-details-pane"), func(i int, x *goquery.Document) {
		word := Word{}
		word.Word = x.Find(".word-text h2").Text()

		pronounces := x.FindMatcher(goquery.Single(".pronounces"))
		word.Katakana = pronounces.Find("span").Text()
		word.AudioUrl = pronounces.FindMatcher(goquery.Single(".word-audio")).AttrOr("data-src", "")

		word.Simple = []SimpleExplain{}

		utils.Each(x.Find(".simple"), func(i int, s *goquery.Document) {
			attributes := s.Find("h2")
			if attributes.Size() == 0 {
				if explains := reSum(x.Find(".simple").Text()); explains != "" {
					word.Simple = append(word.Simple, SimpleExplain{Explains: []string{explains}})
				}
				return
			}

			list := s.Find("ul")
			utils.Each(attributes, func(i int, s *goquery.Document) {
				simple := SimpleExplain{}
				simple.Attribute = s.Text()
				utils.Each(list.Eq(i).Find("li"), func(i int, s *goquery.Document) {
					simple.Explains = append(simple.Explains, s.ReplaceWith("span").Text())
				})
				word.Simple = append(word.Simple, simple)
			})
		})

		word.Detail = getDetails(x)

		words = append(words, word)
	})
	return words
}

func getDetails(x *goquery.Document) (d []Detail) {
	utils.Each(x.Find(".word-details-pane-content .word-details-item"), func(i int, s *goquery.Document) {
		source := s.Find(".detail-source").Text()
		utils.Each(s.Find(".word-details-item-content .detail-groups dl"), func(i int, s *goquery.Document) {
			detail := getDetail(s)
			detail.Source = source
			d = append(d, detail)
		})
	})
	return
}

func getDetail(s *goquery.Document) Detail {
	detail := Detail{}
	detail.Attribute = reSum(s.FindMatcher(goquery.Single("dt")).Text())
	utils.Each(s.Find("dd"), func(i int, s *goquery.Document) {
		explain := strings.Builder{}
		utils.Each(s.Find("h3 p"), func(i int, s *goquery.Document) {
			explain.WriteString(reSum(s.Text()))
		})

		explainsAndExample := ExplainsAndExample{Explain: explain.String()}

		utils.Each(s.Find("ul li"), func(i int, s *goquery.Document) {
			explainsAndExample.Example = append(
				explainsAndExample.Example,
				[2]string{
					reSum(s.FindMatcher(goquery.Single(".def-sentence-from")).Text()),
					reSum(s.FindMatcher(goquery.Single(".def-sentence-to")).Text()),
				},
			)
		})

		detail.ExplainsAndExample = append(detail.ExplainsAndExample, explainsAndExample)
	})

	return detail
}

func GetJson(str string) (string, error) {
	s, err := json.MarshalIndent(Get(str), "", " ")
	return string(s), err
}

func GetCNJson(str string) (string, error) {
	s, err := json.MarshalIndent(GetCN(str), "", " ")
	return string(s), err
}

func FormatString(str string) string {
	return convertToString(Get(str))
}

func FormatCNString(str string) string {
	return convertToString(GetCN(str))
}

func convertToString(y []Word) string {
	s := strings.Builder{}
	for i := range y {
		if i != 0 {
			s.WriteByte('\n')
		}

		s.WriteString(y[i].Word)
		s.WriteByte(' ')
		s.WriteString(y[i].Katakana)
		s.WriteByte(' ')
		s.WriteString(y[i].AudioUrl)
		s.WriteByte('\n')

		for i2 := range y[i].Simple {
			if i2 == 0 {
				s.WriteString("simple explain:\n")
			}
			if y[i].Simple[i2].Attribute != "" {
				s.WriteString(" word attribute:")
				s.WriteString(y[i].Simple[i2].Attribute)
				s.WriteByte('\n')
			}
			for i3 := range y[i].Simple[i2].Explains {
				s.WriteString("   ")
				s.WriteString(strconv.Itoa(i3 + 1))
				s.WriteByte('.')
				s.WriteString(y[i].Simple[i2].Explains[i3])
				s.WriteByte('\n')
			}
		}

		for i2 := range y[i].Detail {
			if i2 == 0 {
				s.WriteString("More Detail:\n")
			}

			if y[i].Detail[i2].Source != "" {
				s.WriteString(" source: ")
				s.WriteString(y[i].Detail[i2].Source)
				s.WriteByte('\n')
			}

			if y[i].Detail[i2].Attribute != "" {
				s.WriteString(" word attribute: ")
				s.WriteString(y[i].Detail[i2].Attribute)
				s.WriteByte('\n')
			}

			explainsAndExample := y[i].Detail[i2].ExplainsAndExample
			for i3 := range explainsAndExample {
				s.WriteString("  ")
				s.WriteString(strconv.Itoa(i3 + 1))
				s.WriteByte('.')
				s.WriteString(explainsAndExample[i3].Explain)
				s.WriteByte('\n')

				example := explainsAndExample[i3].Example
				for i4 := range example {
					if len(example[i4][0]) != 0 {
						s.WriteString("    ")
						s.WriteString(strconv.Itoa(i4 + 1))
						s.WriteByte(')')
						s.WriteString(example[i4][0])
						s.WriteByte('\n')
					}
					if len(example[i4][1]) != 0 {
						s.WriteString("      ")
						s.WriteString(example[i4][1])
						s.WriteByte('\n')
					}
				}
			}
		}
	}

	return s.String()
}
