from sys import stdout
from os import listdir
from os.path import join
import json


def main():
    root = "./tmp/bench"
    res = []

    for dir in listdir(root):
        dir_path = join(root, dir)
        for file_path in listdir(dir_path):
            if ".json" not in file_path:
                continue

            run_method = file_path[:-len(".json")]
            result = {"runMethod": run_method, "benchmark": dir}

            with open(join(dir_path, file_path)) as f:
                result["hyperfine"] = json.load(f)

            with open(join(dir_path, f"{run_method}.profile")) as f:
                inst_count = f.read()
                result["total_dyn_inst"] = int(inst_count[len("total_dyn_inst: ")])

            res.append(result)

    json.dump(res, stdout)


main()
