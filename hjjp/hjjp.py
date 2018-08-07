#!/usr/bin/env python3
import requests
import lxml.html
import argparse #读取终端参数
from termcolor import colored,cprint#颜色

css_word_voice = 'div.word-info > div.pronounces > span.word-audio'
css_word_text  = 'div.word-info > div.word-text > h2'
css_word_prnounces  = 'div.word-info > div.pronounces'
css_word_prnounces_kata = 'span'
css_word_info='div.simple'
css_word_example='div.word-details-item-content'
voice_switch = 0
language=['jp','jc']
useragent = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/67.0.3396.99 Safari/537.36'

def option():
    parser = argparse.ArgumentParser()
    parser.add_argument('word',type=str,help='need search word',nargs='*')
    parser.add_argument("-v","--voice",help="voice link",action="store_true")
    args = parser.parse_args()
    return args.word,args.voice

def start(args_word,voice_switch):
    for word in args_word:
        n=0
        cprint('\n查询单词为  '+word+'\n','red',attrs=['bold'])
        url = 'https://dict.hjenglish.com/'+language[0]+'/'+language[1]+'/' + word
        tree = lxml.html.fromstring((requests.get(url,headers={'User-Agent':useragent})).text)
        word_prnounces = word_simple(tree)
        word_voice = voice(tree)
        word_example = word_example_f(tree)
        for word_info_temp in tree.cssselect(css_word_info):
            print('\n结果'+str(n+1))
            cprint(word_prnounces[n],'yellow')
            if voice_switch==1:
                cprint('音频链接:   '+word_voice[n],'blue',attrs=['underline','bold'])
            for sub_word_info in word_info_temp.text_content().split():
                cprint(sub_word_info,'cyan')
            
            cprint('\n'+'详细解释:','magenta',attrs=['bold'])
            for sub_word_example in word_example[n]:
                cprint(sub_word_example,'green')

            n+=1

def word_simple(tree):
    word_text=[]
    word_prnounces=[]
    for word_text_temp in tree.cssselect(css_word_text):
        word_text.append(word_text_temp.text_content())

    i=0
    for voice in tree.cssselect(css_word_prnounces):
        word_prnounces.append(word_text[i]+'|假名:'+voice.cssselect(css_word_prnounces_kata)[0].text_content()+'|罗马音:'+voice.cssselect(css_word_prnounces_kata)[1].text_content())
        i+=1
    
    return word_prnounces

def voice(tree):
    word_voice=[]
    for voice in tree.cssselect(css_word_voice):
        word_voice.append(str(voice.attrib['data-src']))

    return word_voice

def word_example_f(tree):
    word_example=[]
    for word_example_temp in tree.cssselect(css_word_example):
        word_example.append(word_example_temp.text_content().split())
        
    return word_example


def main():
    tree = start(option()[0],option()[1])

if __name__ == '__main__':
    main()
