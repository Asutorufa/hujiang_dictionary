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

func Get(word string) (all []*Ktbk) {
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
	for _, n := range x.Find("#mainArea article").Nodes {
		one := &Ktbk{}
		xx := goquery.NewDocumentFromNode(n)
		one.Dict = "「　" + xx.Find("h2").Text() + "　」"
		for _, node := range xx.Find("div section").Nodes {
			tmp := ""
			xx = goquery.NewDocumentFromNode(node)
			n := xx.Find(">div").Nodes
			if len(n) <= 0 {
				//log.Println(re2.ReplaceAllString(xx.Find("div section").Text(),""))
				origin, _ := xx.Html()
				origin = strings.ReplaceAll(origin, "<br/>", "\n")
				origin = strings.ReplaceAll(origin, "</div>", "</div>\n ")
				xx.SetHtml(origin)
				tmp = reSpace.ReplaceAllString(xx.Text(), " ")
				tmp = reEnter.ReplaceAllString(tmp, "\n")
				tmp = reEnterSpace.ReplaceAllString(tmp, "\n ")
				one.Imi = append(one.Imi, tmp)
				continue
			}
			for index, x := range n {
				//log.Println(re2.ReplaceAllString(goquery.NewDocumentFromNode(x).Text(),""))
				if index != 0 {
					tmp += "\n"
				}
				dataOrgTag, _ := goquery.NewDocumentFromNode(x).Attr("data-orgtag")
				if dataOrgTag == "meaning" {
					tmp += " " + strings.ReplaceAll(reSpace.ReplaceAllString(goquery.NewDocumentFromNode(x).Text(), ""), "\n", "")
				} else {
					tmp += "  " + strings.ReplaceAll(reSpace.ReplaceAllString(goquery.NewDocumentFromNode(x).Text(), ""), "\n", "")
				}
			}
			one.Imi = append(one.Imi, tmp)
		}
		all = append(all, one)
	}

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
