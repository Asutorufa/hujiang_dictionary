package main

import (
	"flag"
	"fmt"

	"github.com/Asutorufa/hjjp/en"
	"github.com/Asutorufa/hjjp/jp"
	"github.com/Asutorufa/hjjp/kotobakku"
)

func main() {
	enFlag := flag.String("en", "", "english")
	jpFlag := flag.String("jp", "", "japanese")
	ktbkFlag := flag.String("ktbk", "", "コトバック")
	flag.Parse()
	switch {
	case *jpFlag != "":
		fmt.Println(jp.FormatString(*jpFlag))
	case *enFlag != "":
		en.Show(*enFlag)
	case *ktbkFlag != "":
		kotobakku.Show(*ktbkFlag)
	default:
		return
	}
}
