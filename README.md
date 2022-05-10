[Download](https://github.com/Asutorufa/hujiang_dictionary/releases)

## 使用

```shell
# 英语
hj -en hello
hj -en 你好
# 日语
hj -jp こにちは
# 中日
hj -cnjp 你好
#コトバック
hj -ktbk こら

# add -json output json
hj -json -jp こにちは
```

## API

有两种方式,Get(str string)返回结构体,GetJson(str string)返回json,具体请查看代码自行测试

![jp](https://raw.githubusercontent.com/Asutorufa/hujiang_dictionary/golang/img/jp.png)
![en](https://raw.githubusercontent.com/Asutorufa/hujiang_dictionary/golang/img/en.png)

## others

if use `termux`, please use `termux-chroot` to run golang function and create `/erc/resolv.conf` file, otherwise dns resolve will failed.
