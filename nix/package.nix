{ pkgs, packages }:

let
  src = ../.;
  cargo = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package;
  sourceDateEpoch = "1";

  archiveTargets = pkgs.lib.filterAttrs
    (target: _: pkgs.lib.hasInfix "musl" target || pkgs.lib.hasInfix "darwin" target)
    packages;

  debTargets = pkgs.lib.filterAttrs
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

  mkDeb = target: package:
    let
      arch = targetArch target;
      maintainer = builtins.head cargo.authors;
    in
    pkgs.runCommand "${cargo.name}-${cargo.version}-${arch}.deb" {
      nativeBuildInputs = [ pkgs.dpkg ];
    } ''
      export SOURCE_DATE_EPOCH=${sourceDateEpoch}

      root=$TMPDIR/deb
      mkdir -p $root/DEBIAN
      mkdir -p $root/usr/bin
      mkdir -p $root/usr/share/doc/${cargo.name}
      mkdir -p $root/usr/share/bash-completion/completions
      mkdir -p $root/usr/share/zsh/vendor-completions
      mkdir -p $root/usr/share/fish/vendor_completions.d
      mkdir -p $root/usr/share/elvish/lib
      mkdir -p $root/usr/share/man/man1

      cat > $root/DEBIAN/control <<EOF
      Package: ${cargo.name}
      Version: ${cargo.version}
      Architecture: ${arch}
      Maintainer: ${maintainer}
      Description: ${cargo.description}
      Homepage: ${cargo.homepage}
      Priority: optional
      Section: utils
      EOF

      cp ${package}/bin/${cargo.name}       $root/usr/bin/${cargo.name}
      cp ${src}/README.md                   $root/usr/share/doc/${cargo.name}/
      cp ${src}/assets/completions/hanko.bash $root/usr/share/bash-completion/completions/${cargo.name}
      cp ${src}/assets/completions/_hanko     $root/usr/share/zsh/vendor-completions/
      cp ${src}/assets/completions/hanko.fish $root/usr/share/fish/vendor_completions.d/
      cp ${src}/assets/completions/hanko.elv  $root/usr/share/elvish/lib/
      cp ${src}/assets/manpages/*             $root/usr/share/man/man1/

      find $root -exec touch --no-dereference --date="@$SOURCE_DATE_EPOCH" {} +

      dpkg-deb --build --root-owner-group $root $out
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
  debs = builtins.mapAttrs mkDeb debTargets;
  containerImages = builtins.mapAttrs mkContainerImage debTargets;

  releaseArtifacts =
    let
      allArtifacts = builtins.attrValues archives ++ builtins.attrValues debs;
      copyArtifacts = pkgs.lib.concatMapStrings (a: "cp ${a} $out/${a.name}\n") allArtifacts;
    in
    pkgs.runCommand "${cargo.name}-release-artifacts" { } ''
      mkdir $out
      ${copyArtifacts}
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
  inherit archives debs containerImages releaseArtifacts releaseContainers;
}
