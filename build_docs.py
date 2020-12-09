import subprocess
import os

os.system("mkdir ./book/latest")
os.system("mdbook build --dest-dir './book/latest'")
tags = subprocess.run(["git", "tag"], stdout=subprocess.PIPE).stdout.decode('utf-8')

for tag in tags.split("\n"):
    if tag == "":
        continue
    os.system("git checkout " + tag)
    os.system("mdbook build --dest-dir './book/" + tag + "'")
