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
language=['jp','jc']
useragent = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36'
Cookie = 'HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1; TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1'

def option():
    parser = argparse.ArgumentParser()
    parser.add_argument('word',type=str,help='you don\'t understand word(s) and you want to search',nargs='*')
    parser.add_argument("-v","--voice",help="voice link from hujiang.com",action="store_true")
    parser.add_argument("-m","--markdown",help="print as markdown style(for simplenote,hexo...)",action="store_true")
    args = parser.parse_args()
    return args.word,args.voice,args.markdown

def start(args_word,voice_switch,markdown):
    if markdown:
        print("markdown")
    for word in args_word:
        n=0
        cprint('----------查询单词 '+word+'---------','red',attrs=['bold'])
        url = 'https://dict.hjenglish.com/'+language[0]+'/'+language[1]+'/' + word
        tree = lxml.html.fromstring((requests.get(url,headers={'User-Agent':useragent,'Cookie':Cookie})).text)
        if markdown:
            word_prnounces = word_simple_markdown(tree)
        else:
            word_prnounces = word_simple(tree)
        word_voice = voice(tree)
        word_example = word_example_f(tree)
        if word_prnounces == []:
            cprint('抱歉，没有找到你查的单词结果,请核对拼写是否有误\n','cyan')
        for word_info_temp in tree.cssselect(css_word_info):
            print('\n '+word+' 查询结果 '+str(n+1))
            cprint(word_prnounces[n]+'  ','yellow')
            if voice_switch:
                if markdown:
                    cprint('[读音链接]('+word_voice[n]+')  ','blue',attrs=['underline','bold'])
                else:
                    cprint('读音链接:   '+word_voice[n],'blue',attrs=['underline','bold'])
            for sub_word_info in word_info_temp.text_content().split():
                if markdown:
                    cprint(sub_word_info+'  ','cyan')
                else:
                    cprint(sub_word_info,'cyan')
            
            try:
                if markdown:
                    cprint('``详细解释:``  ','magenta',attrs=['bold'])
                else:
                    cprint('详细解释:','magenta',attrs=['bold'])
                for sub_word_example in word_example[n]:
                    if markdown:
                        cprint(sub_word_example+'  ','green')
                    else:
                        cprint(sub_word_example,'green')
            except IndexError as e:
                cprint('没有详细解释','magenta',attrs=['bold'])

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

def word_simple_markdown(tree):
    word_text=[]
    word_prnounces=[]
    for word_text_temp in tree.cssselect(css_word_text):
        word_text.append(word_text_temp.text_content())

    i=0
    for voice in tree.cssselect(css_word_prnounces):
        word_prnounces.append('**'+word_text[i]+'**  \n假名:'+voice.cssselect(css_word_prnounces_kata)[0].text_content()+'|罗马音:'+voice.cssselect(css_word_prnounces_kata)[1].text_content()+'  ')
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
    start(option()[0],option()[1],option()[2])

if __name__ == '__main__':
    main()
