test-all *args:
    cargo hack nextest run --feature-powerset --no-tests pass {{args}}

check-all *args:
    cargo hack check --feature-powerset {{args}}

fmt *args:
    cargo fmt --all {{args}}
