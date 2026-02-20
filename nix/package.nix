{ pkgs, packages }:

let
  src = ../.;
  cargo = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package;

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
      mkdir -p staging/completions

      cp ${package}/bin/${cargo.name} staging/
      cp ${src}/README.md ${src}/LICENSE ${src}/CHANGELOG.md staging/
      cp ${src}/assets/manpages/* staging/
      cp ${src}/assets/completions/* staging/completions/

      tar -czvf $out -C staging .
    '';

  mkDeb = target: package:
    let
      arch = targetArch target;
      maintainer = builtins.head cargo.authors;
    in
    pkgs.runCommand "${cargo.name}-${cargo.version}-${arch}.deb" {
      nativeBuildInputs = [ pkgs.dpkg ];
    } ''
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

      dpkg-deb --build $root $out
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
in
{
  inherit archives debs containerImages;
}
