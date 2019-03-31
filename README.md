**沪江小d日语词典终端版**  
![](https://img.shields.io/github/license/asutorufa/hujiang_japanese_dict.svg)
![](https://img.shields.io/github/release/asutorufa/hujiang_japanese_dict.svg)
[![codebeat badge](https://codebeat.co/badges/e1408f62-46ae-43b0-920d-e38128dcfd48)](https://codebeat.co/projects/github-com-asutorufa-hujiang_japanese_dict-master)  
1.现在已可使用pip3 install hjjpcj安装(termux可直接使用pip进行安装)  
2.卸载使用 pip3 uninstall hjjpcj  
3.linux amd64 现在可直接使用可执行文件运行 无需安装python3 以及相关库文件    
<!--
4.现在使用git远程提交master分支

~~*(旧)运行bash install.sh安装,手机端termux使用bash termux_install.sh安装*~~  
~~*(旧)运行 bash uninstall.sh或bash termux_uninstall.sh卸载*~~  
-->
所使用到的库  : **lxml cssselect requests termcolor**  
若报错 请确保使用pip3安装了以上库  

使用方法:  
可一次性查找多词,加参数``-v``或``--voice``可显示读音链接,``-h``显示帮助文档  
0.9.8.5更新:  
增加``-m``和``--markdown``参数,用来生成用于markdown文档的格式(截图暂时没有更新)   
例如:  
```
hjjp -v 东京 京都 
```
或者:
```
hjjp 东京 京都 --voice
```
![](https://raw.githubusercontent.com/Asutorufa/hujiang-japanese-dict/master/演示-.png)
<br>termux截图:
![](https://raw.githubusercontent.com/Asutorufa/hujiang-japanese-dict/master/termux演示.png)
**查词结果均来自 沪江小d网页版**
