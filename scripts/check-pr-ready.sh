#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
target_ref="${1:-}"
diff_base="HEAD^"

if [[ -n "$target_ref" ]]; then
  git rev-parse --verify "$target_ref^{commit}" >/dev/null
  diff_base="$(git merge-base "$target_ref" HEAD)"
  merge_output="$(mktemp)"
  trap 'rm -f "$merge_output"' EXIT
  if ! git merge-tree --write-tree "$target_ref" HEAD >"$merge_output"; then
    cat "$merge_output" >&2
    echo "merge conflict detected against $target_ref" >&2
    exit 1
  fi
fi

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm --prefix web/visualizer ci --ignore-scripts
npm --prefix web/visualizer audit --audit-level=high
npm --prefix web/visualizer run build

git diff --check "$diff_base"...HEAD

if git diff --numstat "$diff_base"...HEAD | awk '$1 == "-" || $2 == "-" { found=1 } END { exit !found }'; then
  echo "binary files detected in proposed diff" >&2
  exit 1
fi

if rg -n 'base64|<image([[:space:]>])' README.md SECURITY.md docs crates web examples; then
  echo "embedded binary payload marker detected" >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "working tree is not clean" >&2
  git status --short >&2
  exit 1
fi

echo "PR readiness checks passed${target_ref:+ against $target_ref}"
