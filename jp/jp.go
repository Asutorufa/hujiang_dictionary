package jp

import (
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

type Word struct {
	Word     string `json:"word"`
	Katakana string `json:"katakana"`
	Roma     string `json:"roma"`
	AudioUrl string `json:"audio_url"`
	Simple   string `json:"simple"`
	Detail   string `json:"detail"`
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
	//fmt.Println(string(s))
	x, _ := goquery.ParseString(string(s))

	for index, s := range x.Find(".word-details-pane").HtmlAll() {
		if index != 0 {
			fmt.Println()
		}
		word := &Word{}
		x, _ := goquery.ParseString(s)
		word.Word, word.Katakana, word.AudioUrl = x.Find(".word-text h2").Text(), x.Find(".pronounces span").Text(), x.Find(".pronounces .word-audio").Attr("data-src")
		fmt.Println(x.Find(".word-text h2").Text(), x.Find(".pronounces span").Text(), x.Find(".pronounces .word-audio").Attr("data-src"))
		re, _ := regexp.Compile("\r\n | \n | \r")
		re2, _ := regexp.Compile(" +")
		re3, _ := regexp.Compile("\n。")
		reAll := func(str string) string {
			return re3.ReplaceAllString(re2.ReplaceAllString(re.ReplaceAllString(str, " "), "\n"), "。")
		}
		sb := strings.Builder{}
		for _, x := range strings.Split(reAll(x.Find(".simple").Text()), "\n") {
			if x != "" {
				sb.WriteString(" " + x)
				fmt.Println(" ", x)
			}
		}
		word.Simple = sb.String()

		sb.Reset()
		for _, s := range x.Find(".word-details-pane-content .word-details-item").HtmlAll() {
			x, _ := goquery.ParseString(s)
			fmt.Println("more details:")
			reAll2 := func(str string) string {
				return re3.ReplaceAllString(re2.ReplaceAllString(re.ReplaceAllString(str, ""), ""), "。")
			}
			for _, s := range x.Find(".word-details-item-content .detail-groups dl").HtmlAll() {
				x, _ := goquery.ParseString(s)
				sb.WriteString(" word attribute: " + strings.Replace(reAll2(x.Find("dt").Text()), "\n", "", -1))

				fmt.Println(" word attribute:", strings.Replace(reAll2(x.Find("dt").Text()), "\n", "", -1))
				for index, s := range x.Find("dd").HtmlAll() {
					x, _ := goquery.ParseString(s)

					sb.WriteString("  " + strconv.Itoa(index+1) + "." + strings.Replace(reAll2(x.Find("h3").Text()), "\n", "", -1))
					fmt.Println("  " + strconv.Itoa(index+1) + "." + strings.Replace(reAll2(x.Find("h3").Text()), "\n", "", -1))
					for _, s := range strings.Split(reAll2(x.Find("ul").Text()), "\n") {
						if s != "" {
							sb.WriteString("   " + s)
							fmt.Println("   ", s)
						}
					}
				}
			}
		}
		word.Detail = sb.String()
		words = append(words, word)
	}
	return words
}
