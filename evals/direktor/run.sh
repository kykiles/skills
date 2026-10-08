#!/usr/bin/env bash
# Испытание срабатывания direktor: каждый запрос из queries.tsv — новый `claude -p`
# в проекте из fixture/ с одним скиллом; срабатывание = вызов Skill с direktor за первые 3 хода.
# Использование: run.sh <путь к каталогу скилла> <метка> [прогонов=1] [модель=sonnet]
set -euo pipefail
skill=$(realpath "$1"); label=$2; runs=${3:-1}; model=${4:-sonnet}
here=$(dirname "$(realpath "$0")")
out="$here/results/$label.tsv"; mkdir -p "$here/results"
printf 'expect\trun\tloaded\tcost\tquery\n' > "$out"
tail -n +2 "$here/queries.tsv" | while IFS=$'\t' read -r expect query; do
  for r in $(seq "$runs"); do
    # Свежая копия на каждый прогон: прошлый прогон мог менять файлы.
    work=$(mktemp -d); mkdir -p "$work/.claude/skills"; cp -r "$skill" "$work/.claude/skills/direktor"
    cp -r "$here/fixture/." "$work/"; git -C "$work" init -q
    # --max-turns 3 завершает claude с ошибкой — это ожидаемо, важны только первые ходы.
    res=$( (cd "$work" && timeout 180 claude -p "$query" --model "$model" --max-turns 3 \
      --output-format stream-json --verbose --setting-sources project < /dev/null 2>/dev/null || true) \
      | python3 "$here/parse.py")
    printf '%s\t%s\t%s\t%s\n' "$expect" "$r" "$res" "$query" >> "$out"
    rm -rf "$work"
  done
done
python3 - "$out" <<'PY'
import csv, sys
rows = list(csv.DictReader(open(sys.argv[1]), delimiter='\t'))
ok = sum(r['expect'] == r['loaded'] for r in rows)
cost = sum(float(r['cost']) for r in rows if r['cost'] != '?')
print(f"{sys.argv[1]}: верно {ok}/{len(rows)}, ${cost:.2f}")
for r in rows:
    if r['expect'] != r['loaded']:
        print(f"  ждали {r['expect']}, вышло {r['loaded']}: {r['query']}")
PY
