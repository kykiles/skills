#!/usr/bin/env python3
"""Проверяет Rust-примеры скилла: clippy с pedantic и тесты.

Каждый блок ```rust из SKILL.md и references/*.md становится отдельным
крейтом-библиотекой (edition 2024, dead_code = "allow"): примеры не видят
друг друга и не конфликтуют именами. Все крейты собираются одним workspace,
затем запускаются
  cargo clippy --offline --all-targets -- -D warnings -W clippy::pedantic
  cargo test --offline

Использование: check-examples.py [каталог скилла]
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile

skill = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.path.dirname(__file__), "..", "..", "rust-zdorovogo-cheloveka")
skill = os.path.abspath(skill)

md_files = []
for root, _, files in os.walk(skill):
    md_files += [os.path.join(root, f) for f in sorted(files) if f.endswith(".md")]
md_files.sort()

tmp = tempfile.mkdtemp(prefix="skill-examples-")
block_re = re.compile(r"^```rust\n(.*?)^```", re.S | re.M)
members = []
for md in md_files:
    base = re.sub(r"\W", "_", os.path.splitext(os.path.relpath(md, skill))[0]).lower()
    blocks = block_re.findall(open(md, encoding="utf-8").read())
    for i, code in enumerate(blocks, 1):
        crate = f"{base}_{i:02d}"
        os.makedirs(os.path.join(tmp, crate, "src"))
        with open(os.path.join(tmp, crate, "Cargo.toml"), "w") as f:
            f.write(f'[package]\nname = "{crate}"\nversion = "0.0.0"\n'
                    'edition = "2024"\npublish = false\n\n'
                    '[lints.rust]\ndead_code = "allow"\n')
        with open(os.path.join(tmp, crate, "src", "lib.rs"), "w", encoding="utf-8") as f:
            f.write(f"// {os.path.relpath(md, skill)}, блок {i}\n{code}")
        members.append(crate)

with open(os.path.join(tmp, "Cargo.toml"), "w") as f:
    f.write('[workspace]\nresolver = "3"\nmembers = [\n'
            + "".join(f'    "{m}",\n' for m in members) + "]\n")

failed = False


def run(args):
    global failed
    p = subprocess.run(args, cwd=tmp, capture_output=True, text=True)
    if p.returncode:
        print(f"FAIL {' '.join(args)}\n{p.stdout}{p.stderr}")
        failed = True


run(["cargo", "clippy", "--offline", "--workspace", "--all-targets", "--",
     "-D", "warnings", "-W", "clippy::pedantic"])
run(["cargo", "test", "--offline", "--workspace", "--quiet"])

print(f"{'FAIL' if failed else 'ok'}: {len(members)} примеров из {len(md_files)} файлов, "
      f"временный каталог {tmp}")
if not failed:
    shutil.rmtree(tmp)
sys.exit(1 if failed else 0)
