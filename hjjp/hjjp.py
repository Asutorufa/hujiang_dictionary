#!/usr/bin/env python3
import requests
import lxml.html
import sys #读取终端参数

def get(css,tree):
 word=[]
 i=0
 for div_temp in tree.cssselect(css):
  word_temp = div_temp.text_content()#提取翻译
  word.append(word_temp)
  i=i+1

 return word,i

def print_word(word):
 for word_temp in word:
  print('***********************************')
  for word_tmp in word_temp.split():
   print(word_tmp)

def merge(word_1,word_2,i):#合并翻译与例句
 c=[]
 for n in range(i):
  c.append(word_1[n]+word_2[n])

 return c

def main():
 try:
  url = 'https://dict.hjenglish.com/jp/jc/' + str(sys.argv[1])#终端参数
  css_1 = 'header.word-details-pane-header'
  css_2 = 'div.word-details-item-content'
  r = requests.get(url,headers={'User-Agent':'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/67.0.3396.99 Safari/537.36'})
  tree = lxml.html.fromstring(r.text)
  print_word(merge((get(css_1,tree))[0],(get(css_2,tree))[0],(get(css_1,tree))[1]))
 
 
 except IndexError as e:
  pass


if __name__ == '__main__':
    main()
