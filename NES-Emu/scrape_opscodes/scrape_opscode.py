from lxml import html
import requests
from bs4 import BeautifulSoup
from pydantic import BaseModel
import pandas as pd


url = "https://www.nesdev.org/obelisk-6502-guide/reference.html#INX"


dfs = pd.read_html(url)
print(len(dfs))
# print(dfs)

# いらない表を削除
del dfs[0]
del dfs[::2]

# いらない行を削除
for i in range(len(dfs)):
    dfs[i] = dfs[i].drop(dfs[i].index[[0]])

print(dfs)

# # いらない列を削除
# # for i in range(len(dfs)):
# #     dfs[i] = dfs[i].drop(columns=dfs[i].columns[[0]])

# # CSV用に結合
# df = pd.concat([dfs[0], dfs[1]])
# for i in range(2,len(dfs)):
#     df = pd.concat([df, dfs[i]])

# df.to_csv("data.csv", index=False, header=False)
# dfs = pd.read_html(url)
# print(len(dfs))
# print(dfs[2])

# # いらない表を削除
# del dfs[0]
# del dfs[::2]

# # いらない行を削除
# for i in range(len(dfs)):
#     dfs[i] = dfs[i].drop(dfs[i].index[[0]])

# for i in range(len(opsNames)):
#     dfs[i].insert(0, -1, opsNames[i])

# print(dfs[0])
# # CSV用に結合
# df = pd.concat([dfs[0], dfs[1]])
# for i in range(2,len(dfs)):
#     df = pd.concat([df, dfs[i]])

# df.to_csv("data.csv", index=False, header=False)

# # データ加工
# # 1. $ -> 0x
# # 2. ,(\d) \((.+)\)\n -> ,$1 /* ($2) */\n
# # 3. Zero[ ]+Page -> ZeroPage
# # 4. ZeroPage,([XY]) -> ZeroPage_$1
# # 5. "(.+),([XY])" -> $1_$2
# # 6. (Indirect,X) -> Indirect_X
# # 7. (Indirect)_Y -> Indirect_Y