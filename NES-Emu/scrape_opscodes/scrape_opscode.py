from lxml import html
import requests
from bs4 import BeautifulSoup
from pydantic import BaseModel
import pandas as pd
import re

bytes_table = {
    "Absolute":3,
    "Absolute_X":3,
    "Absolute_Y":3,
    "ZeroPage":2,
    "ZeroPage_X":2,
    "ZeroPage_Y":2,
    "Implied":1,
    "Immediate":2,
    "Indirect_X":2,
    "Indirect_Y":2,
}

addressing_modes = {
    "Accumulator"   :"Accumulator",
    "Absolute"      :"Absolute",
    "Absolute,X"    :"Absolute_X",
    "Absolute,Y"    :"Absolute_Y",
    "ZeroPage"      :"ZeroPage",
    "ZeroPage,X"    :"ZeroPage_X",
    "ZeroPage,Y"    :"ZeroPage_Y",
    "Immediate"     :"Immediate",
    "Relative"      :"Relative",
    "Implied"       :"Implied",
    "Indirect"      :"Indirect",
    "(Indirect,X)"  :"Indirect_X",
    "(Indirect),Y"  :"Indirect_Y",
}

class AddressingMode(BaseModel):
    code: str
    bytes: int
    cycles: int
    calc_cycle_mode: str
    addressingmode: str

class OpCode(BaseModel):
    name: str
    addr: "list[AddressingMode]"

def get_official_opcode_list(url: str) -> "list[OpCode]":
    opsNames = official_opsname_list(url=url)
    addr_infos = get_address_mode_infos(url=url)
    opCodes : "list[OpCode]" = []
    for table, opsname in zip(addr_infos, opsNames):
        opcode = OpCode(
            name=opsname,
            addr=[]
        )
        for row in table.itertuples():
            calc_mode = "None"
            if row[4].__contains__("(+1 if page crossed)"):
                calc_mode = "Page"
            elif row[4].__contains__("branch succeeds"):
                calc_mode = "Branch"
            opcode.addr.append(AddressingMode(
                code = row[2],
                bytes = int(row[3]),
                cycles = int(re.sub("\(.+\)", "", row[4])),
                calc_cycle_mode = calc_mode,
                addressingmode = addressing_modes[row[1].replace(" ", "")]
            ))
        opCodes.append(opcode)
    return opCodes

def get_unofficial_opcode_list(url: str) -> "list[OpCode]":
    opsnames = unofficial_opsname_list(url=url)
    res = request_string(url=url)
    opCodes : "list[OpCode]" = []
    i = 0
    is_reading = False
    for line in res.split("\n"):
        split_line = line.split("|")
        if len(split_line) != 5:
            is_reading = False
            continue
        if split_line[0].__contains__("Addressing") or split_line[0].__contains__("--"):
            is_reading = False
            continue
        
        if not is_reading:
            is_reading = True
            opCodes.append(OpCode(name=opsnames[i], addr=[]))
            i += 1
        
        calc_mode = "None"
        if split_line[4].__contains__("*"):
            calc_mode = "Page"
        opCodes[i-1].addr.append(AddressingMode(
            code=split_line[2],
            bytes=int(split_line[3]),
            cycles=int(split_line[4].replace("-", "0").replace("*", "")),
            calc_cycle_mode=calc_mode,
            addressingmode=addressing_modes[split_line[0].replace(" ", "")]
        ))
    return opCodes

def request_string(url: str) -> str:
    r = requests.get(url=url)
    soup = BeautifulSoup(r.content, "html.parser")
    return str(soup)

def official_opsname_list(url: str) -> "list[str]":
    _page = request_string(url=url)
    lxml_data = html.fromstring(str(_page))
    opsHeaders = lxml_data.xpath("//h3/a")
    opsNames = [ opsHead.attrib.get('name') for opsHead in opsHeaders ]
    return opsNames

def get_address_mode_infos(url: str) -> "list[DataFrame]":
    tables = pd.read_html(url)
    del tables[0]
    del tables[::2]

    for i in range(len(tables)):
        tables[i] = tables[i].drop(tables[i].index[[0]])
    return tables

def unofficial_opsname_list(url: str) -> "list[str]":
    strs : "list[str]" = []
    res = request_string(url=url)
    lines = res.split("\n")
    for i, line in enumerate(lines):
        if line == "=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D=3D":
            strs.append(lines[i-1][5:8])
    return strs
