#!/usr/bin/env bash
set -e

rm -rf build && mkdir build
cd build

yarn nereid-cli fetch-index npm://$NEREID

for FOLDER in $FOLDERS; do
  FILE=$(mktemp)
  curl -o $FILE -L "$URL_PREFIX/$FOLDER.tar.gz"
  OUT=$NAME-$VERSION-$FOLDER
  mkdir $OUT && tar xzvf $FILE -C $OUT
  rm $FILE
  ../node_modules/.bin/nereid-cli build $OUT
done
