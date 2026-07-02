{
  pkgs,
  lib,
  inputs,
  ...
}:
let
  shared_libs = with pkgs; [
    libx11
    libxcursor
    libxi
    libxkbcommon
    vulkan-loader
    wayland
  ];
in
{
  env.LD_LIBRARY_PATH = lib.makeLibraryPath shared_libs;
  # make openGL work on non-NixOS systems
  overlays = [ inputs.nixgl.overlay ];

  packages =
    with pkgs;
    [
      alsa-lib
      binaryen
      clang
      http-server
      libc
      libudev-zero
      libxrandr
      mold
      nixgl.nixGLIntel # should work on any system
      pkg-config
      udev
      vulkan-tools
    ]
    ++ shared_libs;

  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };

  scripts = {
    run.exec = "nixGLIntel cargo run";
    rundyn.exec = "nixGLIntel cargo run --features bevy/dynamic_linking";
    run_release.exec = "nixGLIntel cargo run --release";
  };
}
