{
  inputs,
  cell,
}: let
  inherit (inputs) std self cells;
  inherit (inputs.nixpkgs) pkgs;

  crane = inputs.crane.lib.overrideToolchain cells.repo.rust.toolchain;
in {
  # sane default for a binary package
  default = crane.buildPackage {
    src = std.incl self [
      "${self}/Cargo.lock"
      "${self}/Cargo.toml"
      "${self}/src"
    ];

    buildInputs = [pkgs.openssl pkgs.pkgconfig];
  };
}
