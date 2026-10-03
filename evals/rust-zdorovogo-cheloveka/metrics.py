#!/usr/bin/env python3
"""METRIC-строки для grade.sh: признаки «человечности» правки. На PASS/FAIL не влияют.

Использование: metrics.py <исходник> <рабочая копия> <эталон> <pedantic-исходник> <pedantic-копия>

Каталоги сравниваются целиком, кроме .git и target/. Исходник — состояние
после prepare.sh (коммит + dirty/), поэтому чужие правки в diff не попадают.

Признаки в добавленных строках .rs-файлов:
- dbg — `dbg!`;
- println_outside_main — `println!` вне src/main.rs и src/bin/;
- todo — `TODO`/`FIXME`;
- unwrap_outside_tests — `.unwrap()`/`.expect(` вне тестов;
- int_as — `as` с приведением к целому типу вне тестов (тип источника не
  известен, поэтому считаются все приведения к целым);
- clone — `.clone()` вне тестов.
Тестами считаются файлы в tests/, benches/, examples/ и элементы под
`#[cfg(test)]` (от атрибута до закрывающей скобки).

pedantic_new — предупреждения clippy::pedantic, которых не было в исходнике
(сравниваются файл и текст предупреждения без номера строки).
"""
import difflib
import os
import re
import sys
from collections import Counter

SKIP_DIRS = {".git", "target"}


def files(root):
    out = {}
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for name in filenames:
            path = os.path.join(dirpath, name)
            try:
                with open(path, encoding="utf-8") as f:
                    out[os.path.relpath(path, root)] = f.read().splitlines()
            except (UnicodeDecodeError, OSError):
                out[os.path.relpath(path, root)] = None
    return out


def diff(a_root, b_root):
    """Возвращает (добавлено, удалено, [(файл, номер строки, текст)] добавленных)."""
    a, b = files(a_root), files(b_root)
    added, removed, lines = 0, 0, []
    for name in sorted(set(a) | set(b)):
        old, new = a.get(name) or [], b.get(name) or []
        if old == new:
            continue
        sm = difflib.SequenceMatcher(a=old, b=new, autojunk=False)
        for tag, i1, i2, j1, j2 in sm.get_opcodes():
            if tag == "equal":
                continue
            removed += i2 - i1
            added += j2 - j1
            lines += [(name, j + 1, new[j]) for j in range(j1, j2)]
    return added, removed, lines


def is_test(name, lineno, cache, root):
    if re.match(r"(tests|benches|examples)/", name):
        return True
    if name not in cache:
        cache[name] = test_ranges(files_one(os.path.join(root, name)))
    return any(start <= lineno <= end for start, end in cache[name])


def test_ranges(lines):
    """Диапазоны строк (с 1) элементов под `#[cfg(test)]`: от атрибута до
    закрывающей скобки элемента. Скобки считаются без разбора строк и
    комментариев — для отчёта этого достаточно."""
    ranges = []
    i = 0
    while i < len(lines):
        if lines[i].strip().startswith("#[cfg(test)]"):
            depth, opened, j = 0, False, i
            while j < len(lines):
                depth += lines[j].count("{") - lines[j].count("}")
                opened = opened or "{" in lines[j]
                if (opened and depth <= 0) or (not opened and lines[j].rstrip().endswith(";")):
                    break
                j += 1
            ranges.append((i + 1, j + 1))
            i = j
        i += 1
    return ranges


def files_one(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read().splitlines()
    except OSError:
        return []


def main():
    orig, ws, ref, ped_orig, ped_ws = sys.argv[1:6]
    added, removed, lines = diff(orig, ws)
    ref_added, ref_removed, _ = diff(orig, ref)

    m = Counter()
    cache = {}
    for name, lineno, text in lines:
        if not name.endswith(".rs"):
            continue
        test = is_test(name, lineno, cache, ws)
        m["dbg"] += len(re.findall(r"\bdbg!", text))
        if not (name == "src/main.rs" or name.startswith("src/bin/")):
            m["println_outside_main"] += len(re.findall(r"\bprintln!", text))
        m["todo"] += len(re.findall(r"\b(TODO|FIXME)\b", text))
        if not test:
            m["unwrap_outside_tests"] += len(re.findall(r"\.(unwrap\(\)|expect\()", text))
            m["int_as"] += len(re.findall(r"\bas\s+[ui](8|16|32|64|128|size)\b", text))
            m["clone"] += len(re.findall(r"\.clone\(\)", text))

    def warnings(path):
        with open(path, encoding="utf-8") as f:
            return Counter(line.strip() for line in f if line.strip())

    new_warnings = warnings(ped_ws) - warnings(ped_orig)
    print(f"METRIC pedantic_new {sum(new_warnings.values())}")
    for w in sorted(new_warnings):
        print(f"info pedantic: {w}")
    for key in ("dbg", "println_outside_main", "todo", "unwrap_outside_tests", "int_as", "clone"):
        print(f"METRIC {key} {m[key]}")
    print(f"METRIC added {added}")
    print(f"METRIC removed {removed}")
    ref_size = ref_added + ref_removed
    ratio = (added + removed) / ref_size if ref_size else 0.0
    print(f"METRIC size_vs_reference {ratio:.2f}")


main()
