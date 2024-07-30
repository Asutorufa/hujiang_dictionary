package main

import (
	"flag"
	"fmt"

	"github.com/Asutorufa/hujiang_dictionary/en"
	"github.com/Asutorufa/hujiang_dictionary/jp"
	"github.com/Asutorufa/hujiang_dictionary/kotobakku"
	"github.com/Asutorufa/hujiang_dictionary/kr"
	"github.com/Asutorufa/hujiang_dictionary/weblio"
)

func main() {
	enFlag := flag.String("en", "", "english")
	jpFlag := flag.String("jp", "", "japanese to chinese")
	cnjpFlag := flag.String("cnjp", "", "chinese to japanese")
	krFlag := flag.String("kr", "", "korean")
	ktbkFlag := flag.String("ktbk", "", "コトバック")
	jsonFlag := flag.Bool("json", false, "output json")
	webliFlag := flag.String("weblio", "", "weblio辞書")
	flag.Parse()

	if *jsonFlag {
		var data string
		switch {
		case *jpFlag != "":
			data, _ = jp.GetJson(*jpFlag)
		case *cnjpFlag != "":
			data, _ = jp.GetCNJson(*cnjpFlag)
		case *enFlag != "":
			data, _ = en.GetJson(*enFlag)
		case *ktbkFlag != "":
			data, _ = kotobakku.GetJson(*ktbkFlag)
		case *krFlag != "":
			data, _ = kr.GetJson(*krFlag)
		default:
			return
		}

		fmt.Println(data)
		return
	}

	switch {
	case *jpFlag != "":
		fmt.Println(jp.FormatString(*jpFlag))
	case *cnjpFlag != "":
		fmt.Println(jp.FormatCNString(*cnjpFlag))
	case *enFlag != "":
		fmt.Println(en.FormatString(*enFlag))
	case *ktbkFlag != "":
		fmt.Println(kotobakku.FormatString(*ktbkFlag))
	case *krFlag != "":
		fmt.Println(kr.FormatString(*krFlag))
	case *webliFlag != "":
		fmt.Println(weblio.FormatString(*webliFlag))
	default:
		return
	}
}
