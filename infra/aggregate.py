import json
from sys import stdout
from glob import glob
from os import stat


# main will aggregate all profile info into a single json array.
# It walks a file that looks like:
# tmp
# - bench
# -- <benchmark name>
# ---- run_method.json
# ---- run_method.profile
def main():
    res = []
    jsons = glob("./tmp/bench/*/*.json")
    for file_path in jsons:
        if stat(file_path).st_size == 0:
            continue
        name = file_path.split("/")[-2]
        result = {"runMethod": "brilift", "benchmark": name}
        with open(file_path) as f:
            result["hyperfine"] = json.load(f)
        res.append(result)
    json.dump(res, stdout, indent=2)

main()
