# 终端日语词典  
![](https://img.shields.io/github/license/asutorufa/hujiang_japanese_dict.svg)
![](https://img.shields.io/github/release/asutorufa/hujiang_japanese_dict.svg)
![GitHub top language](https://img.shields.io/github/languages/top/asutorufa/hujiang_japanese_dict.svg)
[![codebeat badge](https://codebeat.co/badges/e1408f62-46ae-43b0-920d-e38128dcfd48)](https://codebeat.co/projects/github-com-asutorufa-hujiang_japanese_dict-master)  

- 安装: `pip3 install hjjpcj`  
- 如果报错请尝试安装库: `pip3 install lxml cssselect requests termcolor`  
- 卸载: `pip3 uninstall hjjpcj`  
- linux amd64 有使用pyinstaller生成的可执行文件运行 不保证所有设备能使用   
<!--
4.现在使用git远程提交master分支

~~*(旧)运行bash install.sh安装,手机端termux使用bash termux_install.sh安装*~~  
~~*(旧)运行 bash uninstall.sh或bash termux_uninstall.sh卸载*~~  
-->
用到的库  : **lxml cssselect requests termcolor**  

使用方法:  
```
hjjp 东京
```
- 可一次性查找多词  
```
hjjp 东京 京都
```
- 加参数``-v``或``--voice``可显示读音链接  
```
hjjp -v 东京 京都
hjjp 东京 京都 --voice
```  
- ``-h``显示帮助文档  
```
hjjp -h
```
- 加``-m``和``--markdown``参数,生成markdown文档的格式   
```
hjjp -m 东京 京都
hjjp 东京 京都 --markdown
```
![](https://raw.githubusercontent.com/Asutorufa/hujiang-japanese-dict/master/演示.png)
**查词结果均来自 沪江小d网页版**
