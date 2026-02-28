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

  idx.extensions = [
    "rust-lang.rust-analyzer"
    "tamasfe.even-better-toml"
  ];
}
