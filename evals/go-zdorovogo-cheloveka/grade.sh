#!/usr/bin/env bash
# Оценивает рабочую копию кейса после работы исполнителя. Рабочая копия не меняется.
# Использование: grade.sh <кейс> <каталог>
set -u
here=$(cd "$(dirname "$0")" && pwd)
case_id=$1
case_dir=$here/cases/$case_id
ws=$(cd "$2" && pwd)

fails=0
ok() { echo "ok   $*"; }
fail() { echo "FAIL $*"; fails=$((fails + 1)); }

MIN_GO=""
CROSS_GOOS=""
API_STABLE=""
[ -f "$case_dir/case.env" ] && . "$case_dir/case.env"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# 1. Файлы, которые исполнитель не должен трогать: go.mod/go.sum,
#    сгенерированный код и чужие незакоммиченные правки.
protected=$(
	cd "$case_dir/project"
	ls go.mod go.sum 2>/dev/null
	grep -rlE '^// Code generated .* DO NOT EDIT\.$' --include='*.go' . | sed 's|^\./||'
	if [ -d "$case_dir/dirty" ]; then (cd "$case_dir/dirty" && find . -type f | sed 's|^\./||'); fi
)
protected=$(echo "$protected" | sort -u)
for f in $protected; do
	expected=$case_dir/project/$f
	[ -f "$case_dir/dirty/$f" ] && expected=$case_dir/dirty/$f
	if cmp -s "$expected" "$ws/$f"; then ok "не изменён $f"; else fail "изменён или удалён $f"; fi
done

# 2. gofmt для изменённых и новых Go-файлов исполнителя.
changed=$(cd "$ws" && { git diff --name-only HEAD; git ls-files --others --exclude-standard; } | grep '\.go$' | sort -u)
unformatted=""
for f in $changed; do
	echo "$protected" | grep -qx "$f" && continue
	[ -f "$ws/$f" ] || continue
	[ -n "$(gofmt -l "$ws/$f")" ] && unformatted="$unformatted $f"
done
if [ -z "$unformatted" ]; then ok "gofmt"; else fail "gofmt:$unformatted"; fi

# 3. Посторонние файлы: исполнитель может добавлять Go-файлы и testdata/,
#    но не отчёты, профили покрытия, логи и бинарники.
extra=$(cd "$ws" && git ls-files --others --exclude-standard | grep -v -E '(\.go$|(^|/)testdata/)' | grep -v -x -F "$protected" | tr '\n' ' ')
if [ -z "$extra" ]; then ok "нет посторонних файлов"; else fail "посторонние файлы: $extra"; fi

# 4. Сборка и статические проверки на копии.
mkdir -p "$tmp/ws"
cp -a "$ws/." "$tmp/ws/"
cd "$tmp/ws"
if out=$(go build ./... 2>&1); then ok "go build"; else fail "go build"; echo "$out" | head -20; fi
if out=$(go vet ./... 2>&1); then ok "go vet"; else fail "go vet"; echo "$out" | head -20; fi
if [ -n "$CROSS_GOOS" ]; then
	if out=$(GOOS=$CROSS_GOOS go vet ./... 2>&1); then ok "GOOS=$CROSS_GOOS go vet"; else fail "GOOS=$CROSS_GOOS go vet"; echo "$out" | head -20; fi
fi
if [ -n "$API_STABLE" ]; then
	"$here/prepare.sh" "$case_id" "$tmp/orig" >/dev/null
	if diff <(cd "$tmp/orig" && go doc -short . 2>&1) <(go doc -short . 2>&1) >"$tmp/api.diff"; then
		ok "экспортируемый API не изменён"
	else
		fail "экспортируемый API изменён"
		head -20 "$tmp/api.diff"
	fi
fi

# 5. Скрытые тесты оценщика вместе с тестами проекта, под -race.
cp -r "$case_dir/grader/." "$tmp/ws/"
# Сторож: тест, который рекурсивно запускает сам себя, исчерпывает процессы
# контейнера. Больше 200 тестовых процессов — останавливаем их и считаем провалом.
(
	while sleep 1; do
		if [ "$(pgrep -c -f '\.test( |$)')" -gt 200 ]; then
			touch "$tmp/forkbomb"
			pkill -9 -f '\.test( |$)'
		fi
	done
) &
watchdog=$!
if go test -race -count=1 -timeout 180s ./... >"$tmp/test.log" 2>&1; then
	ok "go test -race (с оценщиком)"
else
	fail "go test -race (с оценщиком)"
	grep -E -- '^(--- FAIL|FAIL|panic|.*_test\.go:[0-9]+:)' "$tmp/test.log" | head -25
fi
kill "$watchdog" 2>/dev/null
wait "$watchdog" 2>/dev/null
if [ -e "$tmp/forkbomb" ]; then
	fail "тесты порождают неограниченное число процессов (остановлены сторожем)"
fi
if [ -n "$MIN_GO" ]; then
	if GOTOOLCHAIN=$MIN_GO go test -count=1 ./... >"$tmp/min.log" 2>&1; then
		ok "go test на $MIN_GO"
	else
		fail "go test на $MIN_GO"
		head -20 "$tmp/min.log"
	fi
fi

# 6. Объём изменений (для ручного ревью).
cd "$ws"
echo "info diff: $(git diff --shortstat HEAD | sed 's/^ //')"
new=$(git ls-files --others --exclude-standard | tr '\n' ' ')
[ -n "$new" ] && echo "info новые файлы: $new"

if [ "$fails" -eq 0 ]; then echo "RESULT PASS"; else echo "RESULT FAIL ($fails)"; fi
[ "$fails" -eq 0 ]
