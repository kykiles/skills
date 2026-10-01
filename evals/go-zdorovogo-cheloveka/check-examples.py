#!/usr/bin/env python3
"""Проверяет Go-примеры скилла: собирает, прогоняет go vet и go test.

Все блоки ```go одного Markdown-файла попадают в один пакет (каждый блок —
отдельный файл), поэтому примеры могут ссылаться друг на друга. Импорты
добавляет goimports. Блок с первой строкой «// Плохо» — намеренно неверный
пример: он обязан компилироваться, но замечания go vet к нему допустимы.
Блок, начинающийся с `package main`, проверяется отдельно.

Использование: check-examples.py [каталог скилла]
Нужны goimports и GOTOOLCHAIN с версией из GO_VERSION.
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile

GO_VERSION = "1.25"
TOOLCHAIN = "go1.25.0"

skill = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.path.dirname(__file__), "..", "..", "go-zdorovogo-cheloveka")
skill = os.path.abspath(skill)
goimports = shutil.which("goimports") or os.path.expanduser("~/go/bin/goimports")
env = dict(os.environ, GOTOOLCHAIN=TOOLCHAIN, GOFLAGS="-mod=mod")

md_files = []
for root, _, files in os.walk(skill):
    md_files += [os.path.join(root, f) for f in sorted(files) if f.endswith(".md")]

tmp = tempfile.mkdtemp(prefix="skill-examples-")
with open(os.path.join(tmp, "go.mod"), "w") as f:
    f.write(f"module examples\n\ngo {GO_VERSION}\n")

block_re = re.compile(r"^```go\n(.*?)^```", re.S | re.M)
bad_files = set()
total = 0
for md in md_files:
    name = re.sub(r"\W", "_", os.path.splitext(os.path.relpath(md, skill))[0]).lower()
    blocks = block_re.findall(open(md, encoding="utf-8").read())
    for i, code in enumerate(blocks, 1):
        total += 1
        bad = code.lstrip().startswith("// Плохо")
        is_main = code.lstrip().startswith("package main")
        is_test = re.search(r"^func (Test|Benchmark|Fuzz|Example)\w*\(", code, re.M) is not None
        pkg_dir = os.path.join(tmp, f"{name}_main{i}" if is_main else name)
        os.makedirs(pkg_dir, exist_ok=True)
        fname = f"b{i:02d}{'_bad' if bad else ''}{'_test' if is_test else ''}.go"
        path = os.path.join(pkg_dir, fname)
        if not code.lstrip().startswith("package "):
            code = f"package {name}\n\n{code}"
        with open(path, "w", encoding="utf-8") as f:
            f.write(code)
        subprocess.run([goimports, "-w", path], check=True)
        if bad:
            bad_files.add(os.path.relpath(path, tmp))

failed = False


def run(args):
    p = subprocess.run(args, cwd=tmp, env=env, capture_output=True, text=True)
    return p.returncode, p.stdout + p.stderr


code, out = run(["go", "build", "./..."])
if code:
    print("FAIL go build\n" + out)
    failed = True
code, out = run(["go", "vet", "./..."])
if code:
    real = [l for l in out.splitlines()
            if re.match(r"^\S+\.go:\d+", l)
            and not any(l.startswith(b) or l.startswith("./" + b) for b in bad_files)
            and "_bad" not in l.split(":")[0]]
    if real:
        print("FAIL go vet\n" + "\n".join(real))
        failed = True
code, out = run(["go", "test", "-race", "-count=1", "./..."])
if code:
    print("FAIL go test\n" + out)
    failed = True

print(f"{'FAIL' if failed else 'ok'}: {total} примеров из {len(md_files)} файлов, временный каталог {tmp}")
if not failed:
    shutil.rmtree(tmp)
sys.exit(1 if failed else 0)
