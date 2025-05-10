{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";

    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;
        # as before
        buildFeatures = [ "gtk4_8" ];
        nativeBuildInputs = with pkgs; [
          (rust-bin.stable.latest.rust.override {
            extensions = [ "rust-src" "rustfmt" "clippy" "rust-analyzer" ];
          })
          pre-commit
          pkg-config
        ];
        buildInputs = with pkgs; [ lldb ];
        # because we'll use it for both `cargoArtifacts` and `bin`
        commonArgs = { inherit src buildInputs nativeBuildInputs; };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        # remember, `set1 // set2` does a shallow merge:
        bin = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      in with pkgs; {
        packages = {
          # that way we can build `bin` specifically,
          # but it's also the default.
          inherit bin;
          default = bin;
        };
        devShells.default = mkShell {
          # instead of passing `buildInputs` / `nativeBuildInputs`,
          # we refer to an existing derivation here
          inputsFrom = [ bin ];
        };
      });
}
