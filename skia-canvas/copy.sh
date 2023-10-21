#!/usr/bin/env bash
set -eu

rm -rf ../src/dep && mkdir -p ../src/dep
rm -rf ../lib/dep && mkdir -p ../lib/dep
cp -r lib package.json ../src/dep
cp -r lib package.json ../lib/dep