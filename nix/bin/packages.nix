{
  inputs,
  cell,
}: let
  inherit (inputs) std self cells;
  inherit (inputs.nixpkgs) pkgs;

  crane = inputs.crane.lib.overrideToolchain cells.repo.rust;
in {
  cfdyndns = crane.buildPackage {
    src = std.incl self [
      "${self}/Cargo.lock"
      "${self}/Cargo.toml"
      "${self}/src"
    ];

    buildInputs = [pkgs.openssl pkgs.pkg-config];
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [cells.repo.rust];
    meta.description = "CloudFlare Dynamic DNS Client";
  };
}
