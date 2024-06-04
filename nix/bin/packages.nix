{
  inputs,
  cell,
}: let
  inherit (inputs.nixpkgs) pkgs;
  inherit (inputs.cells.repo.rust) toolchain;
  buildRustCrateForPkgs = pkgs:
    pkgs.buildRustCrate.override {
      rustc = toolchain;
      cargo = toolchain;
    };
in {
  cfdyndns =
    (pkgs.callPackage "${inputs.self}/Cargo.nix" {
      inherit buildRustCrateForPkgs pkgs;
      inherit (inputs) nixpkgs;
    })
    .rootCrate
    .build
    .overrideAttrs (_: {
      meta.description = "CloudFlare Dynamic DNS Client";
    });
}
