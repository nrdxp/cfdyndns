{
  description = "CI framework, devshell, and package for the Cloudflare Dynamic DNS Client";

  inputs.std.url = "github:divnix/std/v0.24.0-1";
  inputs.std.inputs.nixpkgs.follows = "nixpkgs";
  inputs.std.inputs.devshell.follows = "devshell";

  inputs.fenix.url = "github:nix-community/fenix";

  inputs.crane.url = "github:ipetkov/crane";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";
  inputs.crane.inputs.flake-compat.follows = "";
  inputs.crane.inputs.rust-overlay.follows = "";

  inputs.devshell.url = "github:numtide/devshell";
  inputs.devshell.inputs.nixpkgs.follows = "nixpkgs";

  inputs.nixpkgs.follows = "fenix/nixpkgs";

  outputs = inputs @ {
    self,
    std,
    ...
  }:
    std.growOn {
      inherit inputs;
      systems = ["x86_64-linux"];
      cellsFrom = ./nix;
      cellBlocks = with std.blockTypes; [
        (installables "packages" {ci.build = true;})
        (devshells "shells" {ci.build = true;})
        (pkgs "rust")
      ];
    } {
      devShells = std.harvest self ["repo" "shells"];
      packages = std.harvest self ["bin" "packages"];
    };

  nixConfig = {
    extra-substituters = [
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };
}
