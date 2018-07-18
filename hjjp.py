#!/usr/bin/env python3
import requests
import lxml.html
import sys #读取终端参数
try:
 url = 'https://dict.hjenglish.com/jp/jc/' + str(sys.argv[1])#终端参数
 r = requests.get(url,headers={'User-Agent':'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/67.0.3396.99 Safari/537.36'})
 tree = lxml.html.fromstring(r.text)
 divsimple = tree.cssselect('header.word-details-pane-header')[0]
 word_temp = divsimple.text_content()
 for word in word_temp.split():
  print (word)
 
 print('\n详细解释/例句:')
 divsimple = tree.cssselect('div.word-details-item-content')[0]
 area2 = divsimple.text_content()
 for word2 in word_temp.split():
  print(word2)

except IndexError as e:
 print('抱歉，没有找到你查的单词结果、请核对拼写是否有误')

