#!/usr/bin/env bash
# Готовит рабочую копию кейса для исполнителя.
# Использование: prepare.sh <кейс> <каталог>
set -eu
here=$(cd "$(dirname "$0")" && pwd)
case_dir=$here/cases/$1
dest=$2

[ -d "$case_dir/project" ] || { echo "нет кейса $1" >&2; exit 2; }
[ -e "$dest" ] && { echo "$dest уже существует" >&2; exit 2; }

mkdir -p "$dest"
cp -r "$case_dir/project/." "$dest/"
git -C "$dest" init -q
git -C "$dest" add -A
git -C "$dest" -c user.name=dev -c user.email=dev@example.com commit -qm "initial"
# Незакоммиченные правки «коллеги».
if [ -d "$case_dir/dirty" ]; then
	cp -r "$case_dir/dirty/." "$dest/"
fi
