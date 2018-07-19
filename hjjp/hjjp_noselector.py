#!/usr/bin/env python3
import requests
import lxml.html
import sys #读取终端参数

language=['jp','jc']
url = 'https://dict.hjenglish.com/'+language[0]+'/'+language[1]+'/' + str(sys.argv[1])#终端参数
css_1 = 'header.word-details-pane-header'
css_2 = 'div.word-details-item-content'
useragent = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/67.0.3396.99 Safari/537.36'

def get(css,tree):
 word=[]
 i=0
 for div_temp in tree.cssselect(css):
  word_temp = div_temp.text_content()#提取翻译
  word.append(word_temp)
  i=i+1

 return word,i

def merge(word_1,word_2,number):#合并翻译与例句
 c=[]
 for n in range(number):
  c.append(word_1[n]+'-------详细解释/例句:--------'+word_2[n])

 return c

def print_word(word):
 i=0
 for word_temp in word:
  i+=1
  print('-----------------------解释'+str(i)+'-------------------------------')
  for word_tmp in word_temp.split():
   print(word_tmp)


def main():
 try:
  tree = lxml.html.fromstring((requests.get(url,headers={'User-Agent':useragent})).text)
  print_word(merge((get(css_1,tree))[0],(get(css_2,tree))[0],(get(css_1,tree))[1]))
 except IndexError as e:
  pass


if __name__ == '__main__':
    main()
