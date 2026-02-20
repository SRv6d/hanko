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

        gitRev = self.shortRev or self.dirtyShortRev or null;

        build = import ./nix/build.nix { inherit pkgs crane rustToolchain gitRev; };

        packaging = import ./nix/package.nix {
          inherit pkgs;
          inherit (build) packages;
        };

      in
      {
        packages = {
          hanko   = build.packages.${build.nativeTarget};
          default = build.packages.${build.nativeTarget};
          rust-toolchain = rustToolchain;
          rust-analyzer  = pkgs.rust-analyzer;
        } // build.packages
          // pkgs.lib.mapAttrs'
            (target: archive: pkgs.lib.nameValuePair "archive-${target}" archive)
            packaging.archives
          // pkgs.lib.mapAttrs'
            (target: deb: pkgs.lib.nameValuePair "deb-${target}" deb)
            packaging.debs
          // pkgs.lib.mapAttrs'
            (target: img: pkgs.lib.nameValuePair "container-${target}" img)
            packaging.containerImages;

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
