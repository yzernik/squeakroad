#!/usr/bin/env bash

# pb_disable_eslint.sh
# you can use it like so:
# ./pb_disable-eslint.sh ./my/path/to/proto/dir

for F in $(ls -A1 $1)
do
echo "Prepending file: $1/$F"
echo '/* eslint-disable */' | cat - $1/$F > temp && mv temp $1/$F
done
