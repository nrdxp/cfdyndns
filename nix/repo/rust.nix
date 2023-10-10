# export fenix toolchain as its own package set
{
  inputs,
  cell,
}: let
  inherit (inputs) fenix;
in
  fenix.packages.fromToolchainFile {
    file = "${inputs.self}/rust-toolchain.toml";
    sha256 = "sha256-rLP8+fTxnPHoR96ZJiCa/5Ans1OojI7MLsmSqR2ip8o=";
  }
