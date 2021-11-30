package utils

import "github.com/PuerkitoBio/goquery"

func Each(s *goquery.Selection, f func(int, *goquery.Document)) {
	for i, n := range s.Nodes {
		f(i, goquery.NewDocumentFromNode(n))
	}
}
