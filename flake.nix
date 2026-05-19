{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
            (_: p: {
              my-rust = p.rust-bin.stable.latest.default.override {
                extensions = [
                  "rust-src"
                  "rust-analyzer"
                ];
                targets = [
                  "x86_64-unknown-linux-gnu"
                  "wasm32-unknown-unknown"
                ];
              };

              rundyn = p.writeShellScriptBin "rundyn" ''
                cargo run --features bevy/dynamic_linking
              '';
            })
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell rec {
          # compilers & linkers & dependecy finding programs
          nativeBuildInputs = with pkgs; [
            clang
            http-server
            mold
            my-rust
            pkg-config
            rundyn
            wasm-bindgen-cli_0_2_121
          ];

          # libraries
          buildInputs = with pkgs; [
            alsa-lib
            libc
            libx11
            libxkbcommon
            udev
            vulkan-loader
            wayland
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
