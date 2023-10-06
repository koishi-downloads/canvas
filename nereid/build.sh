#!/usr/bin/env bash
set -e

yarn nereid-cli fetch-index npm://$NEREID

rm -rf canvas-*
for FOLDER in $FOLDERS; do
  FILE=$(mktemp)
  curl -o $FILE -L https://github.com/$REPO/releases/download/$VERSION/$FOLDER.tar.gz
  mkdir $FOLDER && tar xzvf $FILE -C $FOLDER
  rm $FILE
  yarn nereid-cli build $FOLDER
done
