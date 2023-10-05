#!/usr/bin/env bash
set -eu

REPO="Automattic/node-canvas"
NAME="canvas"
VERSION="v2.11.2"
NODE_ABI_VERSIONS="64 67 72 79 83 102 108 111 115"
OS="linux-glibc-x64 darwin-unknown-x64 win32-unknown-x64"

rm -rf canvas-*
for abi in $NODE_ABI_VERSIONS; do
  for os in $OS; do
    FILE=$(mktemp)
    FOLDER=$NAME-$VERSION-node-v$abi-$os
    URL=https://github.com/$REPO/releases/download/$VERSION/$NAME-$VERSION-node-v$abi-$os.tar.gz
    echo $URL
    curl -o $FILE -L $URL
    mkdir $FOLDER
    tar xzvf $FILE -C $FOLDER
    rm $FILE
    yarn nereid-cli build $FOLDER
  done
done
