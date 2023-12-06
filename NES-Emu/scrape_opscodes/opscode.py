import requests
from bs4 import BeautifulSoup

url = "https://www.nesdev.org/obelisk-6502-guide/reference.html#INX"

r = requests.get(url=url)
soup = BeautifulSoup(r.content, "html.parser")
results = soup.find_all("h3")



header = '''
use crate::cpu::AddressingMode;
use crate::cpu::OpCode;
use crate::cpu::CPU;
'''
