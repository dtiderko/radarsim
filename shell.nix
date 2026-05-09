let
  rust_overlay = import (
    builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"
  );

  pkgs = import <nixpkgs> {
    overlays = [
      rust_overlay
      (_: prev: {
        rundyn = prev.writeShellScriptBin "rundyn" ''
          cargo run --features bevy/dynamic_linking
        '';

        my-rust = prev.rust-bin.selectLatestNightlyWith (
          toolchain:
          toolchain.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "rustc-codegen-cranelift-preview"
            ];
            targets = [
              "x86_64-unknown-linux-gnu"
              "wasm32-unknown-unknown"
            ];
          }
        );
      })
    ];
  };
in
pkgs.callPackage (
  {
    mkShellNoCC,
    writeShellScriptBin,

    rundyn,

    clang,
    mold,
    my-rust,
    pkg-config,

    alsa-lib,
    libc,
    libx11,
    libxkbcommon,
    udev,
    wayland,
  }:
  mkShellNoCC {
    strictDeps = true;

    # host/target agnostic programs
    depsBuildBuild = [
      rundyn
    ];

    # compilers & linkers & dependecy finding programs
    nativeBuildInputs = [
      clang
      mold
      my-rust
      pkg-config
    ];

    # libraries
    buildInputs = [
      alsa-lib
      libc
      libx11
      libxkbcommon
      udev
      wayland
    ];

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
      with pkgs;
      [
        vulkan-loader
        libxkbcommon
      ]
    );

    # RUST_BACKTRACE = 1;
  }
) { }
