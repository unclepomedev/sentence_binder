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

fmt-all: fmt-ts fmt-rs

no-jpn:
    # Fails if any Han/Hiragana/Katakana character is found in tracked sources.
    ! rg --hidden --glob '!.git' '[\p{Han}\p{Hiragana}\p{Katakana}]' .
