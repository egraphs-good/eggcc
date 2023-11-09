from sys import stdout
from os import listdir
from os.path import join
import json


# main will aggregate all profile info into a single json array.
# It walks a file that looks like:
# tmp
# - bench
# -- <benchmark name>
# ---- run_method.json
# ---- run_method.profile
def main():
    root = "./tmp/bench"
    res = []

    for dir in listdir(root):
        dir_path = join(root, dir)

        for file_path in listdir(dir_path):
            # since there is a .profile and a .json file for each bench method
            # only fire on files with .json, and we'll extrapolate the name of the .profile
            if ".json" not in file_path:
                continue

            # get the name of the run method as promised above
            run_method = file_path[:-len(".json")]
            result = {"runMethod": run_method, "benchmark": dir}

            # we can use the file_path to get the hyperfine results, store them in result["hyperfine"]
            with open(join(dir_path, file_path)) as f:
                result["hyperfine"] = json.load(f)

            # we can't use file_path, but we can use the run_method to access the .profile file
            # that is holding the instruction count. It should be a single line like:
            # total_dyn_inst: <num insts>
            with open(join(dir_path, f"{run_method}.profile")) as f:
                inst_file_str = f.read()
                # strip the first piece of the string off to get just the number of instructions
                # casting to an int
                result["total_dyn_inst"] = int(inst_file_str[len("total_dyn_inst: "):])

            res.append(result)

    json.dump(res, stdout)


main()
