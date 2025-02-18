import "just/env.just"
import "asan/Justfile"
import "dummy_libc/Justfile"
import "fuzz/Justfile"
import "gasan/Justfile"
import "qasan/Justfile"
import "runner/Justfile"
import "zasan/Justfile"

build: build_asan build_dummy build_fuzz build_gasan build_qasan build_runner build_zasan

test: test_asan

pretty_rust:
  #!/bin/bash
  cargo fmt

pretty_toml:
  #!/bin/bash
  taplo fmt

pretty: pretty_rust pretty_toml

fix: fix_asan fix_dummy fix_fuzz fix_gasan fix_qasan fix_runner fix_zasan

clippy:
  #!/bin/bash
  cargo clippy

doc:
  #!/bin/bash
  cargo doc

all: fix pretty build test clippy doc
