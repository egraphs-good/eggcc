from glob import glob
import subprocess
import os

TIMEOUT = 60

if __name__ == "__main__":
    files = glob("benchmarks/failing/**/*.bril", recursive=True)
    for file in files:
        if file[-9:] == '_lib.bril':
            continue
        print(file)
        try:
            p = subprocess.run(['cargo', 'run', '--release', file],
                               timeout=TIMEOUT, stdout=subprocess.DEVNULL)
            if p.returncode == 0:
                passing = file.replace('failing', 'passing')
                os.makedirs("/".join(passing.split("/")[:-1]), exist_ok=True)
                os.system(f'mv {file} {passing}')
                print("passed and moved")
            else:
                print("failed")
        except subprocess.TimeoutExpired:
            print(f"timed out after {TIMEOUT} seconds")
