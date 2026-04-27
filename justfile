default:
    @just --list

dev:
    bun tauri dev

# TS =====================================================================================================
fmt-ts:
    bunx biome format --write ./src
    bunx biome check --write ./src

# Rust ==================================================================================================
fmt-rs:
    cargo fmt --manifest-path src-tauri/Cargo.toml
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

test-rs:
    cargo test --manifest-path src-tauri/Cargo.toml

fmt-sql:
    sqruff fix src-tauri/migrations --dialect sqlite
    sqruff lint src-tauri/migrations --dialect sqlite

# General =================================================================================================
fmt-all: fmt-ts fmt-rs fmt-sql

no-jpn:
    # Fails if any Han/Hiragana/Katakana character is found in tracked sources.
    ! rg --hidden --glob '!.git' '[\p{Han}\p{Hiragana}\p{Katakana}]' .
