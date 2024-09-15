{ pkgs ? import <nixpkgs> {
  config.allowUnfree = true;
} }:


pkgs.mkShell rec {
  buildInputs = [
    pkgs.libGL
    pkgs.libxkbcommon
    pkgs.xorg.libxcb
    pkgs.wayland 
    pkgs.openssl
    pkgs.clang
    pkgs.llvm
    pkgs.lld
    pkgs.skia
    pkgs.pkg-config
    pkgs.fontconfig
    pkgs.ngrok
    pkgs.android-tools
    pkgs.aapt
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
