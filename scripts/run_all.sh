#!/bin/bash
# Run all pseudo-lang scripts

cd "$(dirname "$0")/.."

echo "========================================"
echo "Running pseudo-lang test scripts"
echo "========================================"

for f in scripts/*.psudo; do
    echo
    echo "----------------------------------------"
    echo "Running: $f"
    echo "----------------------------------------"
    ./target/release/psudo -f "$f"
done

echo
echo "========================================"
echo "All scripts completed"
echo "========================================"