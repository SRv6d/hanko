{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        hanko = import ./nix/build.nix { inherit pkgs craneLib; };
      in
      {
        packages = {
          inherit hanko;
          rust-toolchain = rustToolchain;
          rust-analyzer  = pkgs.rust-analyzer;

          default = hanko;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-deny
            git
            just
            gh
          ];
        };
      }
    );
}
