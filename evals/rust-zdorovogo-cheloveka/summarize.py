#!/usr/bin/env python3
"""Сводные таблицы результатов: results/<условие>/<кейс>-<n>/grade.log.

1. Доля PASS по кейсам и условиям. Провалы делятся на категории:
   - поведение — скрытые тесты оценщика, сборка, Clippy (в том числе incompatible_msrv);
   - гигиена — cargo fmt, посторонние файлы, изменённые манифест, lockfile,
     чужие правки, публичные элементы.
2. Средние METRIC по условиям (на PASS/FAIL не влияют).

Использование: summarize.py [условие...]
"""
import os
import re
import sys
from collections import defaultdict

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "results")
if not os.path.isdir(root):
    sys.exit(f"нет каталога {root}")
conds = sys.argv[1:] or sorted(os.listdir(root))

BEHAVIOUR = re.compile(r"cargo (test|build|clippy)")
METRIC = re.compile(r"^METRIC (\S+) (\S+)$")


def classify(line):
    return "поведение" if BEHAVIOUR.search(line) else "гигиена"


table = defaultdict(dict)
cats = defaultdict(lambda: defaultdict(int))
metrics = defaultdict(lambda: defaultdict(list))
metric_names = []
cases = set()
for cond in conds:
    d = os.path.join(root, cond)
    runs = defaultdict(list)
    for run in sorted(os.listdir(d)):
        path = os.path.join(d, run, "grade.log")
        if not os.path.isfile(path):
            continue
        with open(path, encoding="utf-8") as f:
            log = f.read()
        case = run.rsplit("-", 1)[0]
        cases.add(case)
        ok = "RESULT PASS" in log
        runs[case].append(ok)
        if not ok:
            for k in {classify(l) for l in log.splitlines() if l.startswith("FAIL ")}:
                cats[cond][k] += 1
        for line in log.splitlines():
            m = METRIC.match(line)
            if m:
                name = m.group(1)
                if name not in metric_names:
                    metric_names.append(name)
                metrics[cond][name].append(float(m.group(2)))
    for case, oks in runs.items():
        table[case][cond] = (sum(oks), len(oks))

print("| Кейс | " + " | ".join(conds) + " |")
print("|------|" + "|".join("---" for _ in conds) + "|")
totals = defaultdict(lambda: [0, 0])
for case in sorted(cases):
    cells = []
    for cond in conds:
        p, n = table[case].get(cond, (0, 0))
        totals[cond][0] += p
        totals[cond][1] += n
        cells.append(f"{p}/{n}" if n else "—")
    print(f"| `{case}` | " + " | ".join(cells) + " |")
print("| **Итого PASS** | " + " | ".join(f"**{p}/{n}**" for p, n in (totals[c] for c in conds)) + " |")
print("| провалы: поведение | " + " | ".join(str(cats[c]["поведение"]) for c in conds) + " |")
print("| провалы: гигиена | " + " | ".join(str(cats[c]["гигиена"]) for c in conds) + " |")

if metric_names:
    print()
    print("| METRIC, среднее на прогон | " + " | ".join(conds) + " |")
    print("|------|" + "|".join("---" for _ in conds) + "|")
    for name in metric_names:
        cells = []
        for cond in conds:
            values = metrics[cond][name]
            cells.append(f"{sum(values) / len(values):.2f}" if values else "—")
        print(f"| `{name}` | " + " | ".join(cells) + " |")
