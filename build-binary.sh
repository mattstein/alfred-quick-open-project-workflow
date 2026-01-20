#!/usr/bin/env bash
set -euo pipefail

if command -v lipo >/dev/null 2>&1; then
  CGO_ENABLED=0 GOOS=darwin GOARCH=amd64 go build -ldflags="-s -w" -o search_amd64 .
  CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -ldflags="-s -w" -o search_arm64 .
  lipo -create -output search search_amd64 search_arm64
  rm -f search_amd64 search_arm64
else
  go build -ldflags="-s -w" -o search .
fi
