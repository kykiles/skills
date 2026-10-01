#!/usr/bin/env bash
# Самопроверка стенда: оценщик принимает эталон (reference/) и проваливает
# исходный код (кроме рефакторинга, где исходник корректен) и каждое
# заведомо сломанное решение из broken/.
# Использование: verify.sh [кейс...]
set -u
here=$(cd "$(dirname "$0")" && pwd)
cases=("$@")
[ ${#cases[@]} -eq 0 ] && cases=($(cd "$here/cases" && ls))
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
bad=0

# expect <кейс> <метка> <pass|fail> [overlay]
expect() {
	local c=$1 label=$2 want=$3 overlay=${4:-}
	local ws=$tmp/$c-$label
	"$here/prepare.sh" "$c" "$ws"
	[ -n "$overlay" ] && cp -r "$overlay/." "$ws/"
	if "$here/grade.sh" "$c" "$ws" >"$ws.log" 2>&1; then got=pass; else got=fail; fi
	if [ "$got" = "$want" ]; then
		echo "ok   $c/$label: $(grep '^RESULT' "$ws.log")"
	else
		echo "BAD  $c/$label: ожидалось $want"
		grep -E '^(FAIL|RESULT)' "$ws.log"
		bad=1
	fi
}

for c in "${cases[@]}"; do
	ORIGINAL_PASSES=""
	[ -f "$here/cases/$c/case.env" ] && . "$here/cases/$c/case.env"
	if [ -n "$ORIGINAL_PASSES" ]; then expect "$c" original pass; else expect "$c" original fail; fi
	expect "$c" reference pass "$here/cases/$c/reference"
	for b in "$here/cases/$c/broken"/*/; do
		[ -d "$b" ] && expect "$c" "broken-$(basename "$b")" fail "$b"
	done
done
exit $bad
