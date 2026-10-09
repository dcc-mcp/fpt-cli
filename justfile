set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]


default:
    @vx just --list

fmt:
    vx cargo fmt --all

fmt-check:
    vx cargo fmt --all -- --check

check:
    vx cargo check --workspace

lint:
    vx cargo clippy --workspace --all-targets -- -D warnings

test:
    vx cargo test --workspace

coverage:
    vx cargo llvm-cov --workspace --lcov --output-path lcov.info

coverage-html:
    vx cargo llvm-cov --workspace --html

hakari-generate:
    vx cargo hakari generate
    vx cargo hakari manage-deps -y

hakari-check:
    vx cargo hakari generate --diff
    vx cargo hakari manage-deps --dry-run
    vx cargo hakari verify

pre-commit-install:
    vx python -m pip install pre-commit
    vx python -m pre_commit install --install-hooks --hook-type pre-commit --hook-type pre-push

pre-commit-run:
    vx python -m pre_commit run --all-files

ci:
    vx just fmt-check
    vx just hakari-check
    vx just check
    vx just lint
    vx just test
    vx just package-skills


package-skills:
    vx uv run python scripts/package_openclaw_skill.py skills dist/skills --all

package-openclaw-skill:
    vx uv run python scripts/package_openclaw_skill.py skills/fpt-cli dist/skills


# ClawHub CLI pin. Keep this at 0.8.0 or newer: the registry rejects `publish`
# unless the client sends `acceptLicenseTerms` (MIT-0) in the payload, and the
# CLI only started sending it in 0.8.0. Older pins fail with
# "MIT-0 license terms must be accepted to publish skills" even though
# `--no-input` leaves no way to accept anything.
# Override with CLAWHUB_CLI_PACKAGE (the CI workflow sets it) to try a
# candidate version without editing this file.
clawhub_cli := env_var_or_default("CLAWHUB_CLI_PACKAGE", "clawhub@0.8.0")

clawhub-sync-dry-run:
    vx npx {{clawhub_cli}} sync --root skills --all --dry-run --no-input

clawhub-sync:
    vx npx {{clawhub_cli}} sync --root skills --all --no-input


build-release *args:
    vx cargo build --release -p fpt-cli {{args}}

build-release-target target:
    vx cargo build --release -p fpt-cli --target {{target}}

release-version:
    vx uv run python scripts/release_metadata.py version

release-matrix:
    vx uv run python scripts/release_metadata.py matrix

verify-release-tag tag:
    vx uv run python scripts/release_metadata.py verify-tag {{tag}}

run *args:
    vx cargo run -p fpt-cli -- {{args}}


capabilities:
    vx cargo run -p fpt-cli -- capabilities --output json
