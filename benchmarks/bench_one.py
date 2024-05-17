import sys
import subprocess

args = sys.argv[1]
for (a, b) in [("false", "false"), 
               ("false", "true"), 
               ("true", "false"), 
               ("true", "true")]:
    subprocess.call(f'cargo run --release {args} --run-mode compile-bril-llvm --optimize-egglog {a} --optimize-bril-llvm {b} -o tmp && hyperfine --warmup 3 --max-runs 100 "./tmp `cat tmp-args`"', shell=True)
    subprocess.call('rm tmp && rm tmp-args', shell=True)