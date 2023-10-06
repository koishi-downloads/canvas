#!/usr/bin/env bash
set -eu

rm -rf ../src/dep && mkdir -p ../src/dep
cp -r index.js lib package.json ../src/dep