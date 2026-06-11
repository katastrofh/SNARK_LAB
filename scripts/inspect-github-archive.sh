#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <github-archive-url>" >&2
  exit 2
fi

repo_root="$(git rev-parse --show-toplevel)"
archive="$(mktemp --suffix=.zip)"
extract_dir="$(mktemp -d)"
snapshot_dir="$(mktemp -d)"
trap 'rm -f "$archive"; rm -rf "$extract_dir" "$snapshot_dir"' EXIT

curl --fail --location --proto '=https' --tlsv1.2 --max-filesize 67108864 --output "$archive" "$1"
ARCHIVE="$archive" DESTINATION="$extract_dir" python3 - <<'PY'
import os
from pathlib import Path
from zipfile import ZipFile

archive = Path(os.environ["ARCHIVE"])
destination = Path(os.environ["DESTINATION"]).resolve()
with ZipFile(archive) as bundle:
    members = bundle.infolist()
    if len(members) > 20_000:
        raise SystemExit("archive contains too many entries")
    if sum(member.file_size for member in members) > 256 * 1024 * 1024:
        raise SystemExit("archive expands beyond 256 MiB")
    for member in members:
        output = (destination / member.filename).resolve()
        if destination not in output.parents and output != destination:
            raise SystemExit(f"unsafe archive path: {member.filename}")
        mode = member.external_attr >> 16
        if mode & 0o170000 == 0o120000:
            raise SystemExit(f"symbolic link rejected: {member.filename}")
    bundle.extractall(destination)
PY

root_count="$(find "$extract_dir" -mindepth 1 -maxdepth 1 -type d | wc -l)"
if [[ "$root_count" -ne 1 ]]; then
  echo "expected one repository root in archive" >&2
  exit 1
fi
archive_root="$(find "$extract_dir" -mindepth 1 -maxdepth 1 -type d -print -quit)"

tar -C "$repo_root" \
  --exclude=.git --exclude=target --exclude=node_modules --exclude=dist \
  -cf - . | tar -C "$snapshot_dir" -xf -

echo "Archive extracted safely. No repository files were overwritten."
echo "Comparison against the current worktree:"
git diff --no-index --stat "$snapshot_dir" "$archive_root" || true
