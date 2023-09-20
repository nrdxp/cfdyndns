# export fenix toolchain as its own package set
{
  inputs,
  cell,
}: let
  inherit (inputs) fenix;
in
  fenix.packages.fromToolchainFile {
    file = "${inputs.self}/rust-toolchain.toml";
    sha256 = "sha256-dxE7lmCFWlq0nl/wKcmYvpP9zqQbBitAQgZ1zx9Ooik=";
  }
