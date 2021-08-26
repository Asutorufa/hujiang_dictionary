package jp

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

type SimpleExplain struct {
	Attribute string   `json:"attribute"`
	Explains  []string `json:"explains"`
}

type ExplainsAndExample struct {
	Explain string     `json:"explain"`
	Example [][]string `json:"example"`
}
type Detail struct {
	Attribute          string                `json:"attribute"`
	ExplainsAndExample []*ExplainsAndExample `json:"explains_and_example"`
}

type Word struct {
	Word     string           `json:"word"`
	Katakana string           `json:"katakana"`
	AudioUrl string           `json:"audio_url"`
	Simple   []*SimpleExplain `json:"simple"`
	Detail   []*Detail        `json:"detail"`
}

func Get(str string) []*Word {
	reEnterSpace, _ := regexp.Compile("\r\n | \n | \r")
	reSpace, _ := regexp.Compile(" +")
	reEnterDot, _ := regexp.Compile("\n。")
	reEnter, _ := regexp.Compile("\n+")
	reSum := func(str string) string {
		return strings.TrimSpace(reEnterDot.ReplaceAllString(reSpace.ReplaceAllString(reEnterSpace.ReplaceAllString(reEnter.ReplaceAllString(str, "\n"), ""), ""), "。"))
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
	req, err := http.NewRequest(http.MethodGet, "https://dict.hjenglish.com/jp/jc/"+url.PathEscape(str), nil)
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
	//s, err := ioutil.ReadAll(resp.Body)
	//if err != nil {
	//	log.Println(err)
	//}
	x, _ := goquery.NewDocumentFromReader(resp.Body)

	wait := make(chan bool)
	for _, s := range x.Find(".word-details-pane").Nodes {
		word := &Word{}
		x := goquery.NewDocumentFromNode(s)

		go func() {
			word.Word, word.Katakana = x.Find(".word-text h2").Text(), x.Find(".pronounces span").Text()
			word.AudioUrl, _ = x.Find(".pronounces .word-audio").Attr("data-src")
			wait <- true
		}()

		go func() {
			word.Simple = []*SimpleExplain{}
			for _, s := range x.Find(".simple").Nodes {
				simpleTmpAll := reSum(x.Find(".simple").Text())
				x := goquery.NewDocumentFromNode(s)
				if len(x.Find("h2").Nodes) == 0 {
					if simpleTmpAll == "" {
						break
					}
					simpleTmp := &SimpleExplain{}
					simpleTmp.Attribute = ""
					simpleTmp.Explains = append(simpleTmp.Explains, simpleTmpAll)
					word.Simple = append(word.Simple, simpleTmp)
					break
				}
				list := x.Find("ul").Nodes
				for index, s := range x.Find("h2").Nodes {
					simpleTmp := &SimpleExplain{}
					simpleTmp.Attribute = goquery.NewDocumentFromNode(s).Text()
					x := goquery.NewDocumentFromNode(list[index])
					for _, s := range x.Find("li").Nodes {
						x := goquery.NewDocumentFromNode(s)
						simpleTmp.Explains = append(simpleTmp.Explains, x.Text())
					}
					word.Simple = append(word.Simple, simpleTmp)
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
					detailTmp := &Detail{}
					detailTmp.Attribute = reSum(x.Find("dt").Text())
					for _, s := range x.Find("dd").Nodes {
						x := goquery.NewDocumentFromNode(s)
						explainsAndExampleTmp := &ExplainsAndExample{}
						explainsAndExampleTmp.Explain = strings.Replace(reSum(x.Find("h3").Text()), "\n", "", -1)
						for _, s := range x.Find("ul li").Nodes {
							x := goquery.NewDocumentFromNode(s)
							from := reSum(x.Find(".def-sentence-from").Text())
							to := reSum(x.Find(".def-sentence-to").Text())
							tmp := []string{from, to}
							explainsAndExampleTmp.Example = append(explainsAndExampleTmp.Example, tmp)
						}
						detailTmp.ExplainsAndExample = append(detailTmp.ExplainsAndExample, explainsAndExampleTmp)
					}
					word.Detail = append(word.Detail, detailTmp)
				}
			}
			wait <- true
		}()

		<-wait
		<-wait
		<-wait
		words = append(words, word)
	}
	close(wait)
	return words
}

func GetJson(str string) (string, error) {
	s, err := json.MarshalIndent(Get(str), "", " ")
	return string(s), err
}

func Show(str string) {
	y := Get(str)
	for indexY := range y {
		if indexY != 0 {
			fmt.Println("")
		}
		fmt.Println(y[indexY].Word, y[indexY].Katakana, y[indexY].AudioUrl)

		fmt.Println("simple explain:")
		for index := range y[indexY].Simple {
			if y[indexY].Simple[index].Attribute != "" {
				fmt.Println(" " + y[indexY].Simple[index].Attribute)
			}
			for _, s := range y[indexY].Simple[index].Explains {
				fmt.Println("   " + s)
			}
		}

		fmt.Println("More Detail:")
		for index := range y[indexY].Detail {
			fmt.Println(" word attribute: " + y[indexY].Detail[index].Attribute)
			tmp := y[indexY].Detail[index].ExplainsAndExample
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
	}
}
