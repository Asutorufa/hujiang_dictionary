package en

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
	Word string
	Katakana string
	Roma string
	AudioEnUrl string
	AudioUsUrl string
	EnglishExplains string
	phrase []string
	synonym []string
	antonym []string
	inflections []string
	Simple []string
	Detail string
}

func Get(str string) []*Word  {
	var words []*Word
	req,err := http.NewRequest(http.MethodGet,"https://dict.hjenglish.com/w/"+url.QueryEscape(str),nil)
	if err != nil {
		log.Println(err)
	}
	req.Header.Add("User-Agent","Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36")
	req.Header.Add("Cookie","HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1")
	client := &http.Client{}
	resp,err := client.Do(req)
	if err != nil{
		log.Println(err)
	}
	s,err := ioutil.ReadAll(resp.Body)
	if err != nil{
		log.Println(err)
	}
	//fmt.Println(string(s))
	x, _ := goquery.ParseString(string(s))

	for index,s := range x.Find(".word-details-pane").HtmlAll(){
		if index != 0{
			fmt.Println()
		}
		word := &Word{}
		x, _ := goquery.ParseString(s)
		word.Word = x.Find(".word-text h2").Text()
		if x.Find(".word-info .pronounces .pronounce-value-en").Text() == ""{
			word.Katakana,word.AudioEnUrl =x.Find(".pronounces span").Text(),x.Find(".pronounces .word-audio").Attr("data-src")
			word.AudioUsUrl  = word.AudioEnUrl
		}else{
			//let word_audio_en = "英 " + sub$('.word-info .pronounces .pronounce-value-en').text() + ' ' + sub$('.word-info .pronounces .word-audio-en').attr('data-src');
			//let word_audio_us = "美 " + sub$('.word-info .pronounces .pronounce-value-us').text() + ' ' + sub$('.word-info .pronounces .word-audio').last().attr('data-src');
			word.AudioEnUrl = "英 " + x.Find(".word-info .pronounces .pronounce-value-en").Text()+" "+x.Find(".word-info .pronounces .word-audio-en").Attr("data-src")
			word.AudioUsUrl = "美 " + x.Find(".word-info .pronounces .pronounce-value-us").Text()+" "+x.Find(".word-info .pronounces .word-audio").Last().Attr("data-src")
		}

		re,_ := regexp.Compile("\r\n | \n | \r")
		re2,_ := regexp.Compile(" +")
		reAll := func(str string) string{
			return re2.ReplaceAllString(re.ReplaceAllString(str," ")," ")
		}
		reAll2 := func(str string) string{
			return strings.TrimSpace(re2.ReplaceAllString(strings.Replace(re.ReplaceAllString(str,""),"\n","",-1)," "))
		}
		sb := strings.Builder{}

		if x.Find(".simple p .simple-definition a").Text() == ""{
			for _,s := range x.Find(".simple p").HtmlAll() {
				x, _ := goquery.ParseString(s)
				word.Simple = append(word.Simple,reAll2(x.Text()))
			}
		}else{
			for _,s := range x.Find(".simple p .simple-definition a").HtmlAll(){
				x,_ := goquery.ParseString(s)
				word.Simple = append(word.Simple,reAll2(x.Text()))
			}
		}
		sb.Reset()

		for _,s := range x.Find(".word-details-pane-content .word-details-item").HtmlAll() {
			x, _ := goquery.ParseString(s)
			for _,s := range x.Find(".word-details-item-content .detail-groups dl").HtmlAll(){
				x, _ := goquery.ParseString(s)
				sb.WriteString(" word attribute: "+reAll2(x.Find("dt").Text())+"\n")

				//fmt.Println(" word attribute:",strings.Replace(reAll2(x.Find("dt").Text()),"\n","",-1))
				for index,s := range x.Find("dd").HtmlAll(){
					x, _ := goquery.ParseString(s)
					if index != 0{
						sb.WriteString("\n"+strconv.Itoa(index+1)+"."+strings.TrimSpace(reAll2(x.Find("h3").Text())))
					}else{
						sb.WriteString(strconv.Itoa(index+1)+"."+strings.TrimSpace(reAll2(x.Find("h3").Text())))
					}
					//fmt.Println("  "+strconv.Itoa(index+1)+"."+strings.TrimSpace(reAll2(x.Find("h3").Text())))
					//fmt.Println("  "+strconv.Itoa(index+1)+"."+strings.Replace(reAll2(x.Find("h3").Text()),"\n","",-1))
					for _,s := range x.Find("ul li").HtmlAll(){
						x,_ := goquery.ParseString(s)
						eg := reAll2(x.Find(".def-sentence-from").Text())
						eg2 := reAll2(x.Find(".def-sentence-to").Text())
						sb.WriteString("    \n"+eg+"   \n"+eg2)
					}
				}
			}
		}
		word.Detail = sb.String()

		for _,s := range x.Find(".word-details-item-content .phrase-items li").HtmlAll() {
			x,_ := goquery.ParseString(s)
			word.phrase = append(word.phrase,reAll2(x.Text()))
		}


		sb.Reset()
		for index,s := range x.Find(".word-details-item-content .enen-groups dl").HtmlAll() {
			x,_ := goquery.ParseString(s)
			if index != 0{
				sb.WriteString("\n word attribute: "+reAll2(x.Find("dt").Text())+"\n")
			}else{
				sb.WriteString(" word attribute: "+reAll2(x.Find("dt").Text())+"\n")
			}
			for index,s := range x.Find("dd").HtmlAll() {
				x,_ := goquery.ParseString(s)
				if index !=0{
					sb.WriteString("\n  "+strconv.Itoa(index+1)+"."+reAll(x.Text()))
				}else{
					sb.WriteString("  "+strconv.Itoa(index+1)+"."+reAll(x.Text()))
				}
			}
		}
		word.EnglishExplains = sb.String()


		for _,s := range x.Find(".word-details-item-content .inflections-items li").HtmlAll() {
			x,_ := goquery.ParseString(s)
			word.inflections = append(word.inflections,reAll2(x.Text()))
		}

		for _,s := range x.Find(".word-details-item-content .syn table tbody").HtmlAll() {
			x,_ := goquery.ParseString(s)
			word.synonym = append(word.synonym,reAll2(x.Text()))
		}

		for _,s := range x.Find(".word-details-item-content .ant table tbody").HtmlAll() {
			x,_ := goquery.ParseString(s)
			word.antonym = append(word.antonym,reAll2(x.Text()))
		}

		words = append(words,word)
	}
	return words
}