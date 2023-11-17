import requests
from bs4 import BeautifulSoup

url = "https://www.nesdev.org/obelisk-6502-guide/reference.html#INX"

r = requests.get(url=url)
soup = BeautifulSoup(r.content, "html.parser")
results = soup.find_all("h3")

with open("instruction.txt", "w") as f:
    for row in results:
        print(row.text)
        f.write(row.text + "\n")
