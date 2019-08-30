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
console.log(await require('./hjenAPI').word('tsunami'))
}
test()
```
return a json format,like this:
```json
{
 "word": "tsunami",
 "word_katakana": "",
 "word_audio_en": "英 [tsʊˈnæmɪ] https://tts.hjapi.com/en-gb/76902B674D6DF5B3",
 "word_audio_us": "美 [tsuˈnɑmi] https://tts.hjapi.com/en-us/76902B674D6DF5B3",
 "simple_explain": [
  "1) n. 海啸 "
 ],
 "more_details": {
  " n. /tsʊˈnɑːmɪ/ ": {
   "1. （由海底地震、沉降或火山喷发引起的）海啸 【地理】 ": [],
   "2. 急剧增长；激增 ": []
  }
 },
 "inflections": [
  " 复数: tsunamis "
 ],
 "synonym": [
  " increase "
 ],
 "antonym": [],
 "English_explains": {
  "n.": [
   "a cataclysm resulting from a destructive sea wave caused by an earthquake or volcanic eruption"
  ]
 },
 "phrase": []
}
```

![](https://raw.githubusercontent.com/Asutorufa/hujiang_japanese_dict/nodejs/hj_dict.png)  
**查词结果均来自 沪江小d网页版**
