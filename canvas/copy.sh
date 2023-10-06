#!/usr/bin/env bash
set -eu

rm -rf ../src/dep && mkdir -p ../src/dep
cp -r lib package.json ../src/dep