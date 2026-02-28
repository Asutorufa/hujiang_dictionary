{ pkgs, ... }: {
  packages = [
    pkgs.rustup
    pkgs.gcc
    pkgs.gnumake
    pkgs.binutils
    pkgs.pkg-config
    pkgs.openssl
    pkgs.util-linux.bin
    pkgs.htop
  ];


  # Sets environment variables in the workspace
  env = {
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

    OPENSSL_DIR = "${pkgs.openssl.dev}";
    OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
    OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
  };

  idx.extensions = [
    "rust-lang.rust-analyzer"
    "tamasfe.even-better-toml"
    "google.gemini-cli-vscode-ide-companion"
  ];
}
