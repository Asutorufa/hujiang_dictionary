package main

import (
	"flag"
	"github.com/Asutorufa/hjjp/en"
	"github.com/Asutorufa/hjjp/jp"
)

func main() {
	enFlag := flag.String("en", "", "english")
	jpFlag := flag.String("jp", "", "japanese")
	flag.Parse()
	switch {
	case *jpFlag != "":
		jp.Show(*jpFlag)
	case *enFlag != "":
		en.Show(*enFlag)
	default:
		return
	}
}
