# 沪江终端词典  
[![](https://img.shields.io/github/license/asutorufa/hujiang_japanese_dict.svg)](https://raw.githubusercontent.com/Asutorufa/hujiang_japanese_dict/master/LICENSE)
[![](https://img.shields.io/github/release/asutorufa/hujiang_japanese_dict.svg)](https://github.com/Asutorufa/hujiang_japanese_dict/releases)
![GitHub top language](https://img.shields.io/github/languages/top/asutorufa/hujiang_japanese_dict.svg)

运行:
```shell
npm install
# 日语
node hjjp.js <word>
# 英语
node hjen.js <word>
```
可以链接到~/.local/bin直接运行:
```shell
ln -s $(pwd)/bin/hjjp ~/.local/bin/hjjp
hjjp 東京
ln -s $(pwd)/bin/hjen ~/.local/bin/hjen
hjen hello
hjen 你好
```

打包:
```shell
npm install
npm run pkg 
```

使用方法:  
```
# 日语
node hjjp.js 東京
# 英语
node hjen.js hello
node hjen.js 你好
```

## for developer:
use hjen API:
```javascript
async test(){
console.log(await require('./hjenAPI').word('karaoke'))
}
test()
```
return a json format,like this:
```json
{
 "word": "karaoke",
 "word_katakana": "",
 "word_audio_en": "英 [ˌkɑːrəˈəʊkɪ] https://tts.hjapi.com/en-gb/4E63BA7951A53A8C",
 "word_audio_us": "美 [ˌkæriˈoʊki] https://tts.hjapi.com/en-us/4E63BA7951A53A8C",
 "simple_explain": [
  "1) n. 卡拉OK "
 ],
 "more_details": {
  " n. /ˌkærɪˈəʊkɪ/ ": {
   "1. 卡拉OK ": [
    {
     "eg": " I cannot sing, although I have tried karaoke a few times. ",
     "eg2": " 尽管我试过几次唱卡拉OK，但我还是不会唱。 "
    }
   ]
  },
  " mod. /ˌkærɪˈəʊkɪ/ ": {
   "1. 卡拉OK的 ": [
    {
     "eg": " a karaoke version of Kylie Minogue's hit ",
     "eg2": " 凯莉•米洛名曲的卡拉OK版 "
    }
   ]
  }
 },
 "inflections": [],
 "synonym": [],
 "antonym": [],
 "English_explains": {
  "n.": [
   "singing popular songs accompanied by a recording of an orchestra (usually in bars or nightclubs)"
  ]
 },
 "phrase": []
}
```

![](https://raw.githubusercontent.com/Asutorufa/hujiang_japanese_dict/nodejs/hj_dict.png)  
**查词结果均来自 沪江小d网页版**
