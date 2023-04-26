from osgeo import gdal
import os
import sys

# sys.argv属性にはpythonコマンドで実行するスクリプトに
# 与えられたコマンドライン引数が渡される（スクリプトファイル名を含む）
print("sys.argv:", sys.argv)

script_file_name = sys.argv[0]
print("script file name:", script_file_name)

cmdline_option0 = sys.argv[1]
print("command line option[0]:", cmdline_option0)

path = sys.argv[1]
