#!/usr/bin/env python3
from lxml import etree
f = open('kodomo.html')
selector=etree.HTML(f.read())
#i=0
#for x in selector.xpath('//header[@class="word-details-header"]/ul/li'):
#    i+=1
#    print(selector.xpath('//header[@class="word-details-header"]/ul/li['+str(i)+']/h2/text()'))
#    print(selector.xpath('//header[@class="word-details-header"]/ul/li['+str(i)+']/div[@class="pronounces"]/span[@class="pronounce-value"]/text()'))
   

print(selector.xpath('//section[@class="word-details-content"]'))
