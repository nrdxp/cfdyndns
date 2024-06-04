{
  description = "CI framework, devshell, and package for the Cloudflare Dynamic DNS Client";

  inputs.std.url = "github:divnix/std/v0.24.0-1";
  inputs.std.inputs.nixpkgs.follows = "nixpkgs";
  inputs.std.inputs.devshell.follows = "devshell";

  inputs.fenix.url = "github:nix-community/fenix";

  inputs.devshell.url = "github:numtide/devshell";
  inputs.devshell.inputs.nixpkgs.follows = "nixpkgs";

  inputs.nixpkgs.follows = "fenix/nixpkgs";

  inputs.nosys.follows = "std/paisano/nosys";

  outputs = inputs @ {
    self,
    std,
    nosys,
    ...
  }: let
    systems = ["x86_64-linux"];
  in
    std.growOn {
      inherit inputs systems;
      cellsFrom = ./nix;
      cellBlocks = with std.blockTypes; [
        (installables "packages" {ci.build = true;})
        (devshells "shells" {ci.build = true;})
        (pkgs "rust")
      ];
    } {
      devShells = std.harvest self ["repo" "shells"];
      packages = std.harvest self ["bin" "packages"];
    }
    (nosys (inputs // {inherit systems;}) ({self, ...}: {
      packages.default = self.packages.cfdyndns;
      devShells.default = self.devshells.dev;
    }));

  nixConfig = {
    extra-substituters = [
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };
}
