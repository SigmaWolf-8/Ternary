#!/usr/bin/env bash
ARGS=()
for arg in "$@"; do
    [[ "$arg" == "-Zjson-target-spec" ]] && continue
    ARGS+=("$arg")
done
exec "${RUSTC_REAL:-rustc}" "${ARGS[@]}"
