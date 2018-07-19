#!/usr/bin/env python3
import requests
import lxml.html
import sys #读取终端参数

def main():
 try:
  url = 'https://dict.hjenglish.com/jp/jc/' + str(sys.argv[1])#终端参数
  r = requests.get(url,headers={'User-Agent':'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/67.0.3396.99 Safari/537.36'})
  tree = lxml.html.fromstring(r.text)
  a=[]
  b=[]
  c=[]
  i=0
  for div_temp1 in tree.cssselect('header.word-details-pane-header'):
   word_temp1 = div_temp1.text_content()#提取翻译
   a.append(word_temp1)
   i=i+1
  
  for div_temp2 in tree.cssselect('div.word-details-item-content'):
   word_temp2 = div_temp2.text_content()#提取例句
   b.append(word_temp2)
  
  for n in range(i):
   c.append(a[n]+b[n])#合并翻译与例句
  
  for word_temp3 in c:
   print('***分割线***')
   for word in word_temp3.split():
    print(word)
 
 
 except IndexError as e:
  print('抱歉，没有找到你查的单词结果、请核对拼写是否有误')


if __name__ == '__main__':
    main()
