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
Cookie = 'UZT_USER_SET_106_0_DEFAULT=2%7C94203fe9fb690808b7ef29aff3834b76; HJ_UID=1f044849-7a03-87e4-aec7-6d8b26ca2eb7; TRACKSITEMAP=3%2C22%2C88%2C; _REF=https%3A%2F%2Fwww.baidu.com%2Fs%3Fwd%3D%E6%AE%BA%E3%81%97%E5%90%88%E3%81%84%E3%83%8F%E3%82%A6%E3%82%B9%26rsv_spt%3D1%26rsv_iqid%3D0x8706eaa7000315b2%26issp%3D1%26f%3D8%26rsv_bp%3D0%26rsv_idx%3D2%26ie%3Dutf-8%26tn%3Dbaiduhome_pg%26rsv_enter%3D0%26rsv_sug3%3D1; _SREF_3=https%253A%252F%252Fwww.google.co.jp%252F; HJ_SID=629a515e-1ad8-3a12-22b1-d75f28a725a7; HJ_SSID_3=51e61b73-3447-2402-0c13-6e94bde3a35c; HJ_CSST_3=0; HJ_CST=0; HJ_CMATCH=1'

def option():
    parser = argparse.ArgumentParser()
    parser.add_argument('word',type=str,help='you don\'t understand word(s) and you want to search',nargs='*')
    parser.add_argument("-v","--voice",help="voice link from hujiang.com",action="store_true")
    args = parser.parse_args()
    return args.word,args.voice

def start(args_word,voice_switch):
    for word in args_word:
        n=0
        cprint('----------查询单词 '+word+'---------','red',attrs=['bold'])
        url = 'https://dict.hjenglish.com/'+language[0]+'/'+language[1]+'/' + word
        tree = lxml.html.fromstring((requests.get(url,headers={'User-Agent':useragent,'Cookie':Cookie})).text)
        word_prnounces = word_simple(tree)
        word_voice = voice(tree)
        word_example = word_example_f(tree)
        if word_prnounces == []:
            cprint('抱歉，没有找到你查的单词结果,请核对拼写是否有误\n','cyan')
        for word_info_temp in tree.cssselect(css_word_info):
            print('\n '+word+' 查询结果 '+str(n+1))
            cprint(word_prnounces[n],'yellow')
            if voice_switch:
                cprint('读音链接:   '+word_voice[n],'blue',attrs=['underline','bold'])
            for sub_word_info in word_info_temp.text_content().split():
                cprint(sub_word_info,'cyan')
            
            try:
                cprint('详细解释:','magenta',attrs=['bold'])
                for sub_word_example in word_example[n]:
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
    start(option()[0],option()[1])

if __name__ == '__main__':
    main()
