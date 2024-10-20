package kotobakku

import (
	"encoding/json"
	"strings"

	"github.com/Asutorufa/hujiang_dictionary/httpclient"
	"github.com/Asutorufa/hujiang_dictionary/utils"
	"github.com/PuerkitoBio/goquery"
	"jaytaylor.com/html2text"
)

type Ktbk struct {
	Dict string   `json:"dict"`
	Imi  []string `json:"imi"`
}

func Get(word string) (all []string) {
	// reSpace, _ := regexp.Compile(" +")
	// reEnter, _ := regexp.Compile("\n+")
	// reEnterSpace, _ := regexp.Compile("(\n )+")
	c, err := httpclient.DefaultClient.Get("https://kotobank.jp/word/" + word)
	if err != nil {
		panic(err)
	}
	x, err := goquery.NewDocumentFromReader(c.Body)
	if err != nil {
		panic(err)
	}

	utils.Each(x.Find("#mainArea article"), func(i int, s *goquery.Document) {
		s.Find("a").Each(func(i int, s *goquery.Selection) { s.SetAttr("href", "") })

		html, err := s.Html()
		if err != nil {
			panic(err)
		}

		str, err := html2text.FromString(html, html2text.Options{
			OmitLinks: true,
			TextOnly:  true,
		})
		if err != nil {
			panic(err)
		}

		all = append(all, str)
	})
	return
}

func GetJson(str string) (string, error) {
	s, err := json.MarshalIndent(Get(str), "", " ")
	return string(s), err
}

func FormatString(word string) string {
	data := Get(word)

	return strings.Join(data, "\n\n---------------------------------------\n---------------------------------------\n\n")
}
