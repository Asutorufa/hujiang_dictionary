package main

import (
	"hjjp/jp"
	"os"
)

func main(){
	if len(os.Args) < 2{
		return
	}
	jp.Get(os.Args[1])
}
