#!/usr/bin/env bash
# publish.sh - Dry-run and publish polyfont crates to crates.io in dependency order.
#
# Usage:
#   ./publish.sh dry-run   # Verify all crates publish without errors
#   ./publish.sh publish   # Actually publish to crates.io (irreversible)
#
# Prerequisites:
#   - cargo login (stores token in ~/.cargo/credentials.toml)
#   - All CI checks passing
#   - Workspace version bumped in Cargo.toml

set -euo pipefail

CRATES=(
    polyfont-core
    polyfont-config
    polyfont-scope
    polyfont-parse
    polyfont-fonts
    polyfont-themes
    polyfont-lsp
    polyfont-cli
)

CMD="${1:-dry-run}"

case "$CMD" in
    dry-run)
        echo "=== Dry-run: verifying all crates can publish ==="
        for crate in "${CRATES[@]}"; do
            echo "--- $crate ---"
            cargo publish --dry-run -p "$crate" --allow-dirty 2>&1 || {
                echo "FAILED: $crate"
                exit 1
            }
        done
        echo "=== All crates pass dry-run ==="
        ;;
    publish)
        echo "=== Publishing to crates.io ==="
        for crate in "${CRATES[@]}"; do
            echo "--- Publishing $crate ---"
            cargo publish -p "$crate" 2>&1 || {
                echo "FAILED: $crate"
                exit 1
            }
            # Wait for crates.io index to propagate
            echo "Waiting 10s for crates.io propagation..."
            sleep 10
        done
        echo "=== All crates published ==="
        ;;
    *)
        echo "Usage: $0 {dry-run|publish}"
        exit 1
        ;;
esac
