package google

import (
	"testing"
)

func TestXxx(t *testing.T) {
	t.Log(Translate("The JSON Formatter was created to help folks with debugging. As JSON data is often output without line breaks to save space, it can be extremely difficult to actually read and make sense of it. This tool hoped to solve the problem by formatting and beautifying the JSON data so that it is easy to read and debug by human beings.", "", "zh"))
	t.Log(Translate("The JSON Formatter was created to help folks with debugging. As JSON data is often output without line breaks to save space, it can be extremely difficult to actually read and make sense of it. This tool hoped to solve the problem by formatting and beautifying the JSON data so that it is easy to read and debug by human beings.", "", "ja"))
}
