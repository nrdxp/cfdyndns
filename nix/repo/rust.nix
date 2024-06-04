# export fenix toolchain as its own package set
{
  inputs,
  cell,
}: let
  inherit (inputs) fenix;

  # you may change "default" to any of "[minimal|default|complete|latest]" for variants
  # see upstream fenix documentation for details
  rustPkgs = builtins.removeAttrs fenix.packages.default ["withComponents" "name" "type"];
in
  if builtins.pathExists "${inputs.self}/rust-toolchain.toml"
  then {
    toolchain = fenix.packages.fromToolchainFile {
      file = "${inputs.self}/rust-toolchain.toml";
      sha256 = "sha256-opUgs6ckUQCyDxcB9Wy51pqhd0MPGHUVbwRKKPGiwZU=";
    };
  }
  else if rustPkgs ? rust-analyzer
  then rustPkgs
  else
    rustPkgs
    // {
      inherit (fenix.packages) rust-analyzer;
      toolchain = fenix.packages.combine [
        (builtins.attrValues rustPkgs)
        fenix.packages.rust-analyzer
      ];
    }
