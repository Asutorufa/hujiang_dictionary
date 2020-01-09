package main

import (
	"flag"
	"hjjp/en"
	"hjjp/jp"
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
