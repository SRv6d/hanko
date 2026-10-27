{ pkgs, packages }:

let
  src = ../.;
  cargo = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package;
  sourceDateEpoch = "1";

  archiveTargets = pkgs.lib.filterAttrs
    (target: _: pkgs.lib.hasInfix "musl" target || pkgs.lib.hasInfix "darwin" target)
    packages;

  containerTargets = pkgs.lib.filterAttrs
    (target: _: pkgs.lib.hasInfix "musl" target)
    packages;

  targetArch = target:
    if pkgs.lib.hasPrefix "x86_64" target then "amd64"
    else if pkgs.lib.hasPrefix "aarch64" target then "arm64"
    else throw "unsupported target: ${target}";

  mkArchive = target: package:
    pkgs.runCommand "${cargo.name}-${cargo.version}-${target}.tar.gz" { } ''
      export SOURCE_DATE_EPOCH=${sourceDateEpoch}

      mkdir -p staging/completions

      cp ${package}/bin/${cargo.name} staging/
      cp ${src}/README.md ${src}/LICENSE ${src}/CHANGELOG.md staging/
      cp ${src}/assets/manpages/* staging/
      cp ${src}/assets/completions/* staging/completions/

      ${pkgs.gnutar}/bin/tar \
        --sort=name \
        --mtime="@$SOURCE_DATE_EPOCH" \
        --clamp-mtime \
        --owner=0 \
        --group=0 \
        --numeric-owner \
        --format=posix \
        --pax-option=delete=atime,delete=ctime \
        --use-compress-program="${pkgs.gzip}/bin/gzip -n" \
        -C staging \
        -cf $out \
        .
    '';

  mkContainerImage = target: package:
    let
      arch = targetArch target;
      maintainer = builtins.head cargo.authors;
    in
    pkgs.dockerTools.buildImage {
      name = cargo.name;
      tag = cargo.version;
      architecture = arch;
      config = {
        Entrypoint = [ "${package}/bin/${cargo.name}" ];
        Labels = {
          "org.opencontainers.image.title" = cargo.name;
          "org.opencontainers.image.authors" = maintainer;
          "org.opencontainers.image.version" = cargo.version;
          "org.opencontainers.image.description" = cargo.description;
          "org.opencontainers.image.source" = cargo.repository;
        };
      };
    };

  archives = builtins.mapAttrs mkArchive archiveTargets;
  containerImages = builtins.mapAttrs mkContainerImage containerTargets;

  releaseArtifacts =
    let
      artifacts = builtins.sort
        (a: b: a.name < b.name)
        (builtins.attrValues archives);
      copyArtifacts = pkgs.lib.concatMapStrings (a: "cp ${a} $out/${a.name}\n") artifacts;
      writeChecksums = pkgs.lib.concatMapStrings (a:
        "${pkgs.coreutils}/bin/sha256sum ${pkgs.lib.escapeShellArg a.name} >> SHA256SUMS\n"
      ) artifacts;
    in
    pkgs.runCommand "${cargo.name}-release-artifacts" { } ''
      mkdir $out
      ${copyArtifacts}

      (
        cd $out
        : > SHA256SUMS
        ${writeChecksums}
      )
    '';

  releaseContainers =
    let
      copyImages = pkgs.lib.concatMapStrings (target:
        "cp ${containerImages.${target}} $out/${targetArch target}.tar.gz\n"
      ) (builtins.attrNames containerImages);
    in
    pkgs.runCommand "${cargo.name}-release-containers" { } ''
      mkdir $out
      ${copyImages}
    '';
in
{
  inherit archives containerImages releaseArtifacts releaseContainers;
}
