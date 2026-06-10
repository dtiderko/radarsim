{ pkgs, lib, ... }:
let
  libs = with pkgs; [
    alsa-lib
    libc
    libx11
    libxkbcommon
    udev
    vulkan-loader
    wayland
  ];
in
{
  env.LD_LIBRARY_PATH = lib.makeLibraryPath libs;

  packages =
    with pkgs;
    [
      # compilers & linkers & dependecy finding programs
      clang
      http-server
      mold
      pkg-config
      binaryen
    ]
    ++ libs;

  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };

  scripts.rundyn.exec = ''
    cargo run --features bevy/dynamic_linking
  '';

  git-hooks.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
  };
}
