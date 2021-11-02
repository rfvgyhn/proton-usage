#!/bin/bash

set -e

version=${1:?"First arg should be version"}
bin="target/release/proton-usage"

[[ ! -f "$bin" ]] && { echo "Need to build --release"; exit 1; }

outdir="artifacts"
release_name="proton-usage_${version}_linux-x64"
staging="$outdir/$release_name"
mkdir -p "$staging"
cp ./{README.md,LICENSE,CHANGELOG.md} "$staging/"
cp "$bin" "$staging/"
tar czf "$staging.tar.gz" -C "$outdir" "$release_name"