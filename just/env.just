# Provides:
#   - `ARCH`: Architecture (either `x86`, `x86_64`, `powerpc`, `arm`, or `aarch64`). Default is x86_64##
#   - `PROFILE`: Profile (either `dev` or `release`). Default is `dev`.
#   - `PROFILE_DIR`: Profile directory (either `debug` or `release`). Default is `debug`
#   - `TARGET_DIR`: target directry. Defaults to `target`.
#   - `DOTENV`: file containing environment variables for the selected architecture

ARCH := env("ARCH", "x86_64")
PROFILE := env("PROFILE", "dev")
PROFILE_DIR := if PROFILE == "dev" { "debug" } else { "release" }
TARGET_DIR := absolute_path(env("TARGET_DIR", source_directory() / ".." / "target"))
DOTENV := source_directory() / "envs" / ".env." + ARCH

