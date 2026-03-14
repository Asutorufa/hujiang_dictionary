import sys

def count_delims(filename):
    with open(filename, 'r') as f:
        content = f.read()

    stack = []
    delims = {')': '(', '}': '{', ']': '['}
    opens = delims.values()

    line_num = 1
    col_num = 1

    for char in content:
        if char == '\n':
            line_num += 1
            col_num = 1
        else:
            if char in opens:
                stack.append((char, line_num, col_num))
            elif char in delims:
                if not stack:
                    print(f"Extra closing delimiter '{char}' at line {line_num}, col {col_num}")
                else:
                    last_open, l, c = stack.pop()
                    if last_open != delims[char]:
                        print(f"Mismatched delimiter: '{char}' at line {line_num}, col {col_num} closes '{last_open}' from line {l}, col {c}")
            col_num += 1

    for char, l, c in stack:
        print(f"Unclosed delimiter '{char}' from line {l}, col {c}")

count_delims('worker/src/lib.rs')
