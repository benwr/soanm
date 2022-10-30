#!/bin/sh
set -xe

target=$(mktemp)
os=$(uname)
arch=$(uname -m)

curl -fsSL "https://github.com/benwr/soanm/releases/download/0.1.1/soanm-$os-$arch" > "$target"
chmod +x "$target"

$target enroll "$@"
