#!/usr/bin/env bash
# Оценивает рабочую копию кейса после работы исполнителя. Рабочая копия не меняется:
# сборка и тесты идут на копии, CARGO_TARGET_DIR — во временном каталоге.
# Использование: grade.sh <кейс> <каталог>
set -u
here=$(cd "$(dirname "$0")" && pwd)
case_id=$1
case_dir=$here/cases/$case_id
ws=$(cd "$2" && pwd)

fails=0
ok() { echo "ok   $*"; }
fail() { echo "FAIL $*"; fails=$((fails + 1)); }

API_STABLE=""
[ -f "$case_dir/case.env" ] && . "$case_dir/case.env"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
export CARGO_TERM_COLOR=never
cargo=(cargo --offline --locked)

# Копия дерева без .git и target/.
copy() {
	mkdir -p "$2"
	tar -C "$1" --exclude=./target --exclude=./.git -cf - . | tar -C "$2" -xf -
}

# 1. Файлы, которые исполнитель не должен трогать: манифест, lockfile,
#    конфигурация инструментов и чужие незакоммиченные правки.
protected=$(
	cd "$case_dir/project"
	ls Cargo.toml Cargo.lock .gitignore rust-toolchain rust-toolchain.toml \
		rustfmt.toml .rustfmt.toml clippy.toml .clippy.toml 2>/dev/null
	if [ -d "$case_dir/dirty" ]; then (cd "$case_dir/dirty" && find . -type f | sed 's|^\./||'); fi
)
protected=$(echo "$protected" | sort -u)
for f in $protected; do
	expected=$case_dir/project/$f
	[ -f "$case_dir/dirty/$f" ] && expected=$case_dir/dirty/$f
	if cmp -s "$expected" "$ws/$f"; then ok "не изменён $f"; else fail "изменён или удалён $f"; fi
done

# 2. Посторонние файлы: исполнитель может добавлять .rs-файлы,
#    но не отчёты, логи, бэкапы и бинарники.
extra=$(cd "$ws" && git ls-files --others --exclude-standard | grep -v -E '\.rs$' | grep -v -x -F "$protected" | tr '\n' ' ')
if [ -z "$extra" ]; then ok "нет посторонних файлов"; else fail "посторонние файлы: $extra"; fi

# Исходное состояние (коммит + dirty/) — для API, METRIC и размера правки.
"$here/prepare.sh" "$case_id" "$tmp/orig-git" >/dev/null
copy "$tmp/orig-git" "$tmp/orig"
copy "$ws" "$tmp/ws"
cd "$tmp/ws"

# 3. cargo fmt --check только для изменённых и новых .rs-файлов исполнителя:
#    чужой неотформатированный файл — не его забота.
changed=$(cd "$ws" && { git diff --name-only HEAD; git ls-files --others --exclude-standard; } | grep '\.rs$' | sort -u)
unformatted=""
fmt_out=$(cargo fmt --all --check 2>&1)
for f in $changed; do
	echo "$protected" | grep -qx "$f" && continue
	[ -f "$ws/$f" ] || continue
	echo "$fmt_out" | grep -q "^Diff in $tmp/ws/$f:" && unformatted="$unformatted $f"
done
if [ -z "$unformatted" ]; then ok "cargo fmt --check"; else fail "cargo fmt --check:$unformatted"; fi

# 4. Сборка и Clippy (-D warnings включает incompatible_msrv по rust-version из манифеста).
export CARGO_TARGET_DIR=$tmp/target
if out=$("${cargo[@]}" build --all-targets 2>&1); then ok "cargo build"; else fail "cargo build"; echo "$out" | grep -E '^(error|warning)|-->' | head -20; fi
if out=$("${cargo[@]}" clippy --all-targets -- -D warnings 2>&1); then
	ok "cargo clippy -D warnings"
else
	fail "cargo clippy -D warnings"
	echo "$out" | grep -E '^(error|warning)|-->' | head -20
fi

if [ -n "$API_STABLE" ]; then
	crate=$(sed -n 's/^name *= *"\(.*\)"/\1/p' Cargo.toml | head -1 | tr - _)
	api() { grep -o 'href="[^"]*\.html"' "$1/doc/$crate/all.html" | sort; }
	(cd "$tmp/orig" && CARGO_TARGET_DIR=$tmp/target-orig "${cargo[@]}" doc --no-deps --lib -q) >/dev/null 2>&1
	"${cargo[@]}" doc --no-deps --lib -q >/dev/null 2>&1
	if diff <(api "$tmp/target-orig") <(api "$tmp/target") >"$tmp/api.diff" 2>&1; then
		ok "публичные элементы не изменены (cargo doc)"
	else
		fail "публичные элементы изменены (cargo doc)"
		head -20 "$tmp/api.diff"
	fi
fi

# METRIC: предупреждения clippy::pedantic до и после (без тестов оценщика).
pedantic() {
	(cd "$1" && CARGO_TARGET_DIR=$2 "${cargo[@]}" clippy --all-targets --message-format=short -- -W clippy::pedantic 2>&1) |
		grep -E '^[^ ]+:[0-9]+:[0-9]+: warning: ' | sed -E 's/:[0-9]+:[0-9]+: /: /'
}
pedantic "$tmp/orig" "$tmp/target-orig" >"$tmp/pedantic-orig.txt"
pedantic "$tmp/ws" "$tmp/target" >"$tmp/pedantic-ws.txt"

# 5. Скрытые тесты оценщика вместе с тестами проекта.
cp -r "$case_dir/grader/." "$tmp/ws/"
if timeout 600 "${cargo[@]}" test >"$tmp/test.log" 2>&1; then
	ok "cargo test (с оценщиком)"
else
	fail "cargo test (с оценщиком)"
	grep -E -A3 '^(error(\[|:)|---- |thread .* panicked)' "$tmp/test.log" | grep -v -E '^(--|note: run with)' | head -40
	grep -E '(memory allocation of|signal: |process didn.t exit successfully)' "$tmp/test.log" | head -5
fi

# 6. METRIC — оценка «человечности» правки, на PASS/FAIL не влияет.
copy "$tmp/orig" "$tmp/ref"
cp -r "$case_dir/reference/." "$tmp/ref/"
python3 "$here/metrics.py" "$tmp/orig" "$ws" "$tmp/ref" "$tmp/pedantic-orig.txt" "$tmp/pedantic-ws.txt"

if [ "$fails" -eq 0 ]; then echo "RESULT PASS"; else echo "RESULT FAIL ($fails)"; fi
[ "$fails" -eq 0 ]
