{
  description = "Development environment for abc";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};

        # Stable toolchain for building and testing
        stableToolchain = fenix.packages.${system}.stable.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
        ];

        # Nightly toolchain for formatting with unstable features
        nightlyToolchain = fenix.packages.${system}.latest.withComponents [
          "cargo"
          "rustfmt"
        ];
        # Wrapper for nightly cargo that ensures nightly tools are used
        cargo-nightly = pkgs.writeShellApplication {
          name = "cargo-nightly";
          runtimeInputs = [ nightlyToolchain ];
          text = ''
            export CARGO_HOME=/nonexistent
            exec cargo "$@"
          '';
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            stableToolchain
            nightlyToolchain
            cargo-nightly
            pkgs.just
          ];
        };
      }
    );
}
