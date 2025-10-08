{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    cargo
    rustc
    rustfmt
    rust-analyzer
    ra-multiplex

    # System dependencies
    pkg-config
    openssl
    alsa-lib
    udev
    wayland
    wayland-protocols
    libxkbcommon
    vulkan-loader
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
      pkgs.libxkbcommon
      pkgs.wayland
      pkgs.udev
      pkgs.alsa-lib
      pkgs.vulkan-loader
    ]}"
    export WINIT_UNIX_BACKEND=wayland
  '';
}
