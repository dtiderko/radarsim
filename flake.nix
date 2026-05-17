{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      advisory-db,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
            targets = [
              "x86_64-unknown-linux-gnu"
              "wasm32-unknown-unknown"
            ];
          }
        );
        src = craneLib.cleanCargoSource ./.;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;

          # compilers & linkers & dependecy finding programs
          nativeBuildInputs = with pkgs; [
            clang
            mold
            pkg-config
          ];

          # libraries
          buildInputs = with pkgs; [
            alsa-lib
            libc
            libx11
            libxkbcommon
            udev
            wayland
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        crate = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
        # scripts
        rundyn = pkgs.writeShellScriptBin "rundyn" ''
          cargo run --features bevy/dynamic_linking
        '';
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit crate;

          # check code style
          crate-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          # check docs
          crate-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );

          # check formatting
          crate-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          crate-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };
        };

        packages.default = crate;
        apps.default = flake-utils.lib.mkApp {
          drv = crate;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # additional packages
          packages = with pkgs; [
            rundyn
            http-server
            wasm-bindgen-cli_0_2_121
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            with pkgs;
            [
              vulkan-loader
              libxkbcommon
            ]
          );
        };
      }
    );
}
