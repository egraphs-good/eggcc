from glob import glob
import os

if __name__ == "__main__":
    files = glob("benchmarks/failing/polybench/**/*.bril", recursive=True)
    for file in files:
        if file[-9:] == '-int.bril' or file[-9:] == '_lib.bril':
            continue
        print(file)
        new_file = file.replace('.bril', '-int.bril')
        with open(file) as f:
            text = f.read()
            if 'sqrt' in text:
                continue
            text = text.replace('float', 'int')
            text = text.replace('fadd', 'add')
            text = text.replace('fsub', 'sub')
            text = text.replace('fmul', 'mul')
            text = text.replace('fdiv', 'div')
            text = text.replace('feq', 'eq')
            text = text.replace('fle', 'le')
            text = text.replace('flt', 'lt')
            text = text.replace('fge', 'ge')
            text = text.replace('fgt', 'gt')
            text = text.replace('1.5', '3')
            text = text.replace('1.2', '2')
            with open(new_file, 'w') as g:
                g.write(text)     
        os.system(f'bril2json -p < {new_file} | brilift -j > /dev/null')   