#!/usr/bin/env bash
set -eu

cd build
../node_modules/.bin/nereid-cli pub $NEREID
