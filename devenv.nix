{ pkgs, ... }:
{
  packages = with pkgs; [
    # compilers & linkers & dependecy finding programs
    clang
    http-server
    mold
    pkg-config
    wasm-bindgen-cli_0_2_121

    # libraries
    alsa-lib
    libc
    libx11
    libxkbcommon
    udev
    vulkan-loader
    wayland
  ];

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
