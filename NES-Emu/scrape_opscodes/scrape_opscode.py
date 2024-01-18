from lxml import html
import requests
from bs4 import BeautifulSoup
from pydantic import BaseModel
import pandas as pd

url = "https://www.nesdev.org/obelisk-6502-guide/reference.html#INX"

r = requests.get(url=url)
soup = BeautifulSoup(r.content, "html.parser")
lxml_data = html.fromstring(str(soup))

with open("Reference.html", "w", encoding='utf-8') as f:
    f.write(str(soup))

class AddressingMode(BaseModel):
    code: str
    bytes: int
    cycles: int
    addressingmode: str

class OpCode(BaseModel):
    name: str
    addr: list[AddressingMode]

opsHeaders = lxml_data.xpath("//h3/a")
# for opsHead in opsHeaders:
#     print(opsHead.attrib.get('name'))

opsNames = [ opsHead.attrib.get('name') for opsHead in opsHeaders ]
print(opsNames)
dfs = pd.read_html(url)
print(len(dfs))
print(dfs[2])

# いらない表を削除
del dfs[0]
del dfs[::2]

# いらない行を削除
for i in range(len(dfs)):
    dfs[i] = dfs[i].drop(dfs[i].index[[0]])

# いらない列を削除
# for i in range(len(dfs)):
#     dfs[i] = dfs[i].drop(columns=dfs[i].columns[[0]])

# CSV用に結合
df = pd.concat([dfs[0], dfs[1]])
for i in range(2,len(dfs)):
    df = pd.concat([df, dfs[i]])

df.to_csv("data.csv", index=False, header=False)
dfs = pd.read_html(url)
print(len(dfs))
print(dfs[2])

# いらない表を削除
del dfs[0]
del dfs[::2]

# いらない行を削除
for i in range(len(dfs)):
    dfs[i] = dfs[i].drop(dfs[i].index[[0]])

for i in range(len(opsNames)):
    dfs[i].insert(0, -1, opsNames[i])

print(dfs[0])
# CSV用に結合
df = pd.concat([dfs[0], dfs[1]])
for i in range(2,len(dfs)):
    df = pd.concat([df, dfs[i]])

df.to_csv("data.csv", index=False, header=False)

# データ加工
# 1. $ -> 0x
# 2. ,(\d) \((.+)\)\n -> ,$1 /* ($2) */\n
# 3. Zero[ ]+Page -> ZeroPage
# 4. ZeroPage,([XY]) -> ZeroPage_$1
# 5. "(.+),([XY])" -> $1_$2
# 6. (Indirect,X) -> Indirect_X
# 7. (Indirect)_Y -> Indirect_Y