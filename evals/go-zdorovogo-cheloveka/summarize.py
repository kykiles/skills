#!/usr/bin/env python3
"""Сводная таблица результатов: results/<условие>/<кейс>-<n>/grade.log.

Провалы делятся на категории:
- поведение — скрытые тесты оценщика, сборка, go vet, проверка на минимальной версии Go;
- гигиена — gofmt, посторонние файлы, изменённые чужие/защищённые файлы, API.

Использование: summarize.py [условие...]
"""
import os
import re
import sys
from collections import defaultdict

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "results")
conds = sys.argv[1:] or sorted(os.listdir(root))

BEHAVIOUR = re.compile(r"go test|go build|go vet|на go1")


def classify(line):
    return "поведение" if BEHAVIOUR.search(line) else "гигиена"


table = defaultdict(dict)
cats = defaultdict(lambda: defaultdict(int))
cases = set()
for cond in conds:
    d = os.path.join(root, cond)
    runs = defaultdict(list)
    for run in sorted(os.listdir(d)):
        log = open(os.path.join(d, run, "grade.log"), encoding="utf-8").read()
        case = run.rsplit("-", 1)[0]
        cases.add(case)
        ok = "RESULT PASS" in log
        runs[case].append(ok)
        if not ok:
            kinds = {classify(l) for l in log.splitlines() if l.startswith("FAIL ")}
            for k in kinds:
                cats[cond][k] += 1
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
