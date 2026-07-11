#!/usr/bin/env bash
set -euo pipefail

IMAGE="${1:?Usage: $0 <docker-image>}"

CID=$(docker create "$IMAGE")
mkdir -p ./src/tests/snapshots
docker cp "$CID:/app/snapshots-out/tests-snapshots/." ./src/tests/snapshots/ 2>/dev/null || true
docker cp "$CID:/app/snapshots-out/test-output.txt" ./test-output.txt 2>/dev/null || true
docker rm "$CID" >/dev/null
echo "Snapshots extracted to tests/snapshots/"
echo "Test output saved to test-output.txt"
