{ pkgs, craneLib }:

let
  src = craneLib.cleanCargoSource ./..;

  commonArgs = {
    inherit src;
    strictDeps = true;

    nativeBuildInputs = with pkgs; [
      pkg-config
    ];

    buildInputs = with pkgs; [
      openssl
    ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
      pkgs.darwin.apple_sdk.frameworks.Security
      pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
    ];

    # vergen-gix in build.rs reads git metadata, which is unavailable in the
    # Nix sandbox. Idempotent mode makes it emit stable placeholder values
    # instead of failing.
    VERGEN_IDEMPOTENT = "1";
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (commonArgs // {
  inherit cargoArtifacts;
})
