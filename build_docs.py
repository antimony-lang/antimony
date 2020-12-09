import subprocess
import os

os.system("mdbook build --dest-dir './book'")
os.system("mdbook build --dest-dir './book/latest'")
tags = subprocess.run(["git", "tag"], stdout=subprocess.PIPE).stdout.decode('utf-8')

for tag in tags.split("\n"):
    if tag == "":
        continue
    print("Building tag", tag)
    os.system("git switch " + tag)
    os.system("mdbook build --dest-dir './book/" + tag + "'")
