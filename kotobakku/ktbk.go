package kotobakku

import (
	"encoding/json"
	"fmt"
	"net/http"
	"regexp"
	"strings"

	"github.com/PuerkitoBio/goquery"
)

type Ktbk struct {
	Dict string   `json:"dict"`
	Imi  []string `json:"imi"`
}

func Get(word string) (all []Ktbk) {
	reSpace, _ := regexp.Compile(" +")
	reEnter, _ := regexp.Compile("\n+")
	reEnterSpace, _ := regexp.Compile("(\n )+")
	c, err := http.Get("https://kotobank.jp/word/" + word)
	if err != nil {
		panic(err)
	}
	x, err := goquery.NewDocumentFromReader(c.Body)
	if err != nil {
		panic(err)
	}

	x.Find("#mainArea article").Each(func(i int, s *goquery.Selection) {
		one := Ktbk{
			Dict: "「　" + s.Find("h2").Text() + "　」",
		}

		s.Find("div section").Each(func(i int, s *goquery.Selection) {
			imi := ""
			n := s.Find(">div")
			if n.Size() == 0 {
				//log.Println(re2.ReplaceAllString(xx.Find("div section").Text(),""))
				origin, _ := s.Html()
				origin = strings.ReplaceAll(origin, "<br/>", "\n")
				origin = strings.ReplaceAll(origin, "</div>", "</div>\n ")
				s.SetHtml(origin)
				imi = reSpace.ReplaceAllString(s.Text(), " ")
				imi = reEnter.ReplaceAllString(imi, "\n")
				imi = reEnterSpace.ReplaceAllString(imi, "\n ")
				one.Imi = append(one.Imi, imi)
				return
			}

			n.Each(func(i int, s *goquery.Selection) {
				//log.Println(re2.ReplaceAllString(goquery.NewDocumentFromNode(x).Text(),""))
				if i != 0 {
					imi += "\n"
				}

				if s.AttrOr("data-orgtag", "") == "meaning" {
					imi += " " + strings.ReplaceAll(reSpace.ReplaceAllString(s.Text(), ""), "\n", "")
				} else {
					imi += "  " + strings.ReplaceAll(reSpace.ReplaceAllString(s.Text(), ""), "\n", "")
				}
			})
			one.Imi = append(one.Imi, imi)
		})

		all = append(all, one)
	})
	return
}

func GetJson(str string) (string, error) {
	s, err := json.MarshalIndent(Get(str), "", " ")
	return string(s), err
}

func Show(word string) {
	data := Get(word)

	for index := range data {
		fmt.Print(data[index].Dict)
		for _, im := range data[index].Imi {
			fmt.Print(im)
		}
		fmt.Println()
		fmt.Println()
	}
}
