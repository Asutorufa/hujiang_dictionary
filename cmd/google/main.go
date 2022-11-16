package main

import (
	"flag"
	"fmt"

	"github.com/Asutorufa/hujiang_dictionary/google"
)

func main() {
	target := flag.String("target", "en", "-target")
	source := flag.String("source", "auto", "-source")
	text := flag.String("text", "", "-text")
	flag.Parse()

	if *text == "" {
		panic("text is empty")
	}

	z, err := google.Translate(*text, *source, *target)
	if err != nil {
		panic(err)
	}

	for i, t := range z.Target {
		fmt.Printf("%d. %s\n", i+1, t)
		fmt.Printf("%d. %s\n\n", i+1, z.Source[i])
	}
}
