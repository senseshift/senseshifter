set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --list

# Fix all warnings and errors
fix:
    cargo fix --allow-dirty --allow-staged --all-features
    cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features -- -D warnings
    cargo fmt --all