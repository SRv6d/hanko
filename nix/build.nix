{ pkgs, crane, rustToolchain, gitRev ? null }:

let
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  src = craneLib.cleanCargoSource ./..;

  nativeTarget = pkgs.stdenv.hostPlatform.rust.rustcTargetSpec;

  # All Linux targets with their nixpkgs cross package sets.
  linuxTargets = {
    "x86_64-unknown-linux-gnu"   = pkgs.pkgsCross.gnu64;
    "aarch64-unknown-linux-gnu"  = pkgs.pkgsCross.aarch64-multiplatform;
    "x86_64-unknown-linux-musl"  = pkgs.pkgsCross.musl64;
    "aarch64-unknown-linux-musl" = pkgs.pkgsCross.aarch64-multiplatform-musl;
  };

  platformTargets =
    if pkgs.stdenv.hostPlatform.isLinux then linuxTargets
    else if pkgs.stdenv.hostPlatform.isDarwin then {
      ${nativeTarget} = pkgs;
      ${if pkgs.stdenv.hostPlatform.isAarch64
        then "x86_64-apple-darwin"
        else "aarch64-apple-darwin"} = pkgs;
    }
    else { };

  # Toolchain extended with cross-compilation targets.
  crossTargetNames =
    builtins.filter (t: t != nativeTarget) (builtins.attrNames platformTargets);
  rustToolchainCross = rustToolchain.override {
    targets = crossTargetNames;
  };
  craneLibCross =
    (crane.mkLib pkgs).overrideToolchain rustToolchainCross;

  mkPackage = target: _targetPkgs:
    let
      isCross = target != nativeTarget;
      targetPkgs = if isCross then _targetPkgs else pkgs;
      craneLib' = if isCross then craneLibCross else craneLib;

      cc = targetPkgs.stdenv.cc;
      crossCc = "${cc}/bin/${cc.targetPrefix}cc";
      targetUnderscored = builtins.replaceStrings [ "-" ] [ "_" ] target;
      targetUpper = pkgs.lib.toUpper targetUnderscored;
      isLinuxCross = isCross && pkgs.lib.hasInfix "linux" target;

      commonArgs = {
        inherit src;
        strictDeps = true;

        nativeBuildInputs = [ pkgs.pkg-config ];

        buildInputs = [
          targetPkgs.openssl
        ] ++ pkgs.lib.optionals targetPkgs.stdenv.hostPlatform.isDarwin [
          targetPkgs.darwin.apple_sdk.frameworks.Security
          targetPkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

      } // pkgs.lib.optionalAttrs (gitRev != null) {
        GIT_SHA = gitRev;
      } // pkgs.lib.optionalAttrs isCross {
        CARGO_BUILD_TARGET = target;
        HOST_CC = "${pkgs.stdenv.cc}/bin/cc";
      } // pkgs.lib.optionalAttrs isLinuxCross {
        "CARGO_TARGET_${targetUpper}_LINKER" = crossCc;
        # cc crate expects CC_<target> with underscores (not uppercased).
        "CC_${targetUnderscored}" = crossCc;
      };

      cargoArtifacts = craneLib'.buildDepsOnly commonArgs;
    in
    craneLib'.buildPackage (commonArgs // {
      inherit cargoArtifacts;
    });

  packages = builtins.mapAttrs mkPackage platformTargets;
in
{
  inherit packages nativeTarget;
}
