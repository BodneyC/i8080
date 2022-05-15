#!/usr/bin/env bash

cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo doc --no-deps
rm -rf ./docs
echo "<meta http-equiv=\"refresh\" content=\"0; url=i8080/index.html\">" > target/doc/index.html
cp -r target/doc ./docs
