from lxml import html
import requests
from bs4 import BeautifulSoup
from pydantic import BaseModel
import pandas as pd

class AddressingMode(BaseModel):
    code: str
    bytes: int
    cycles: int
    addressingmode: str

class OpCode(BaseModel):
    name: str
    addr: "list[AddressingMode]"

def request_string(url: str) -> str:
    r = requests.get(url=url)
    soup = BeautifulSoup(r.content, "html.parser")
    return str(soup)

def official_opsname_list(url: str) -> "list[str]":
    _page = request_string(url=url)
    lxml_data = html.fromstring(str(_page))
    opsHeaders = lxml_data.xpath("//h3/a")
    opsNames = [ opsHead.attrib.get('name') for opsHead in opsHeaders ]
    print(opsNames)
    return opsNames

def get_address_mode_infos(url: str) -> "list[AddressingMode]":
    tables = pd.read_html(url)
    del tables[0]
    del tables[::2]

    for i in range(len(tables)):
        tables[i] = tables[i].drop(tables[i].index[[0]])
    addrs:"list[AddressingMode]" = []
    for table in tables:
        

def official_opcode_list(url: str) -> "list[OpCode]":
    opsNames = official_opsname_list(url)
    # opcode_infos = 

