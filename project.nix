# SPDX-FileCopyrightText: 2025 FreshlyBakedCake
#
# SPDX-License-Identifier: MIT

{
  pins,
  config,
  lib,
}:
{
  config = {
    name = "sprinkles";

    packages.sprinkles = {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      package =
        {
          fenix,
          stdenv,
          lib,
          installShellFiles,
          dbus,
          ...
        }:
        let
          toolchain = fenix.complete.toolchain;

          manifest = (lib.importTOML ./Cargo.toml).package;

          platform = config.inputs.nixos-unstable.result.${stdenv.hostPlatform.system}.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        platform.buildRustPackage {
          meta.mainProgram = "sprinkles";
          pname = manifest.name;
          version = manifest.version;

          src = ./.;

          buildInputs = [ dbus ];
          nativeBuildInputs = [ installShellFiles ];

          cargoLock.lockFile = ./Cargo.lock;
        };
    };

    shells.sprinkles = {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      shell =
        {
          mkShell,
          kdePackages,
          fenix,
          bacon,
          pkg-config,
          reuse,
          dbus,
          sqlx-cli,
          system,
          ...
        }:
        mkShell {
          QML_IMPORT_PATH =
            lib.fp.pipe
              [
                (map (pkg: "${pkg}/lib/qt-6/qml"))
                (builtins.concatStringsSep ":")
              ]
              [
                (config.inputs.quickshell.result.packages.${system}.default.override {
                  gitRev = pins.quickshell.revision;
                })
                kdePackages.qtdeclarative
              ];

          buildInputs = [ dbus ];
          packages = [
            kdePackages.qtdeclarative
            (fenix.complete.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
              "rust-analyzer"
            ])
            sqlx-cli
            bacon
            pkg-config
            reuse
            dbus
          ];
        };
    };

    shells.sprinkles-testing = {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      shell =
        {
          mkShell,
          libnotify,
          sqlitebrowser,
          system,
          ...
        }:
        mkShell {
          buildInputs = [ libnotify ];
          packages = [
            libnotify
            sqlitebrowser
            config.packages.sprinkles.result.${system}
            (config.inputs.quickshell.result.packages.${system}.default.override {
              gitRev = pins.quickshell.revision;
            })
          ];
        };
    };
  };
}
