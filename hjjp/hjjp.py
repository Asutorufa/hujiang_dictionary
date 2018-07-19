#!/usr/bin/env python3
import requests
import lxml.html
import argparse #读取终端参数

voice_switch = 0
language=['jp','jc']
css_1 = 'header.word-details-pane-header'
css_2 = 'div.word-details-item-content'
css_voice = 'span.word-audio.audio.audio-light'
useragent = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/67.0.3396.99 Safari/537.36'

def option():
    parser = argparse.ArgumentParser()
    parser.add_argument('word',type=str,help='need search word',nargs='*')
    parser.add_argument("-v","--voice",help="read word",action="store_true")
    args = parser.parse_args()
    if args.voice:
        voice_switch = 1

    return args.word

def start(args_word):
    i=0
    for word in args_word:
        i+=1
        print('\n########################查询单词('+str(i)+') '+word+' #####################################\n')
        url = 'https://dict.hjenglish.com/'+language[0]+'/'+language[1]+'/' + word
        try:
            tree = lxml.html.fromstring((requests.get(url,headers={'User-Agent':useragent})).text)
            print_word(merge((get(css_1,tree))[0],(get(css_2,tree))[0],(get(css_1,tree))[1]))
        except IndexError as e:
            pass

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
  c.append(word_1[n]+'**************详细解释/例句:*************'+word_2[n])

 return c

def print_word(word):
 i=0
 for word_temp in word:
  i+=1
  print('-----------------------解释'+str(i)+'-------------------------------')
  for word_tmp in word_temp.split():
   print(word_tmp)


def main():
    start(option())

if __name__ == '__main__':
    main()
