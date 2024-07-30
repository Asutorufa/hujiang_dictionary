package weblio

import (
	"strings"

	"github.com/Asutorufa/hujiang_dictionary/httpclient"
	"github.com/Asutorufa/hujiang_dictionary/utils"
	"github.com/PuerkitoBio/goquery"
	"jaytaylor.com/html2text"
)

// https://www.weblio.jp/content/%E4%BB%8A%E6%97%A5%E3%81%AF

func Get(str string) ([]string, error) {
	c, err := httpclient.DefaultClient.Get("https://www.weblio.jp/content/" + str)
	if err != nil {
		return nil, err
	}
	x, err := goquery.NewDocumentFromReader(c.Body)
	if err != nil {
		return nil, err
	}

	all := []string{}
	utils.Each(x.Find(".kiji"), func(i int, s *goquery.Document) {
		s.Find(".footNote").Each(func(i int, s *goquery.Selection) { s.Remove() })
		s.Find(".footNoteB").Each(func(i int, s *goquery.Selection) { s.Remove() })
		s.Find("a").Each(func(i int, s *goquery.Selection) { s.SetAttr("href", "") })

		html, err := s.Html()
		if err != nil {
			all = append(all, err.Error())
			return
		}

		str, err := html2text.FromString(html, html2text.Options{
			OmitLinks: true,
			TextOnly:  true,
		})

		if err != nil {
			all = append(all, err.Error())
			return
		}

		all = append(all, str)
	})

	return all, nil
}

func FormatString(word string) string {
	data, err := Get(word)
	if err != nil {
		return err.Error()
	}

	return strings.Join(data, "\n\n---------------------------------------\n---------------------------------------\n\n")
}
