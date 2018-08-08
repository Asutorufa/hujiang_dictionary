#!/usr/bin/env python3
#-*- coding:utf-8 -*-

from setuptools import setup, find_packages

setup(
    name = "hjjpcj",
    version = "0.9.8",
    keywords = ("japanese dict dictionary hujiang"),
    description = "hujiang dictionary",
    long_description = "a dictionary from hujiang to search japanese word",
    license = "MIT Licence",

    url = "https://github.com/Asutorufa/hujiang-japanese-dict",
    author = "Asutorufa",
    author_email = "pip@just2hide.anyalias.com",
    entry_points={'console_scripts':['hjjp = hjjp.hjjp:main']},

    packages = find_packages(),
    include_package_data = True,
    platforms = "any",
    install_requires = []
)
