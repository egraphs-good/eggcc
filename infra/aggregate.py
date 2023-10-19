from sys import stdout
from os import listdir
from os.path import join
import json

root="./tmp/bench"

res = []

for dir in listdir(root):
    dir_path = join(root, dir)
    for file_path in listdir(dir_path):
        run_method = file_path[:-len(".json")]
        with open(join(dir_path, file_path)) as f:
            result = {"runMethod": run_method, "benchmark": dir, "results": json.load(f)}
            res.append(result)

json.dump(res, stdout)