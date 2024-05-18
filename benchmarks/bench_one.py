import sys
import subprocess

args = sys.argv[1]
for (a, b) in [("false", "false"), 
               ("false", "true"), 
               ("true", "false"), 
               ("true", "true")]:
    subprocess.call(f'cargo run --release {args} --run-mode llvm --optimize-egglog {a} --optimize-bril-llvm {b} -o tmp && hyperfine --warmup 3 --max-runs 100 "./tmp `cat tmp-args`"', shell=True)
    if not (len(sys.argv) > 2 and sys.argv[2] == "-o"):
        subprocess.call('rm tmp && rm tmp-args', shell=True)