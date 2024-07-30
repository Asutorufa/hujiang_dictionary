package kotobakku

import (
	"fmt"
	"log"
	"regexp"
	"testing"
)

func TestShow(t *testing.T) {
	//Show("こら")
	//Show("API")
	//Show("蟻")
	fmt.Println(FormatString("蟻"))
}

func TestGet(t *testing.T) {
	x, _ := regexp.Compile("(\n )+")
	ss := "sss\n \n \n \n dddd \n \n \n cc"
	log.Println(x.ReplaceAllString(ss, "\n "))
}
