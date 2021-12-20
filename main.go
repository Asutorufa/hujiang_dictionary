package main

import (
	"flag"
	"fmt"

	"github.com/Asutorufa/hujiang_dictionary/en"
	"github.com/Asutorufa/hujiang_dictionary/jp"
	"github.com/Asutorufa/hujiang_dictionary/kotobakku"
	"github.com/Asutorufa/hujiang_dictionary/kr"
)

func main() {
	enFlag := flag.String("en", "", "english")
	jpFlag := flag.String("jp", "", "japanese")
	krFlag := flag.String("kr", "", "korean")
	ktbkFlag := flag.String("ktbk", "", "コトバック")
	flag.Parse()
	switch {
	case *jpFlag != "":
		fmt.Println(jp.FormatString(*jpFlag))
	case *enFlag != "":
		fmt.Println(en.FormatString(*enFlag))
	case *ktbkFlag != "":
		kotobakku.Show(*ktbkFlag)
	case *krFlag != "":
		fmt.Println(kr.FormatString(*krFlag))
	default:
		return
	}
}
