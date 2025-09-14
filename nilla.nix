# SPDX-FileCopyrightText: 2025 FreshlyBakedCake
#
# SPDX-License-Identifier: MIT

let
  pins = import ./npins;

  nilla = import pins.nilla;
in
nilla.create ({ config, lib }: {
  config = {
    inputs = {
      fenix.src = pins.fenix;
      quickshell.src = pins.quickshell;

      nixpkgs = {
        src = pins.nixpkgs;

        settings = {
          overlays = [
            config.inputs.fenix.result.overlays.default
          ];
        };
      };
    };

    packages.default = config.packages.sprinkles;
    packages.sprinkles = {
      systems = [ "x86_64-linux" "aarch64-linux" ];

      package = { fenix, makeRustPlatform, lib, installShellFiles, dbus, ... }:
        let
          toolchain = fenix.complete.toolchain;

          manifest = (lib.importTOML ./Cargo.toml).package;

          platform = makeRustPlatform {
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

    shells.default = config.shells.sprinkles;
    shells.sprinkles = {
      systems = [ "x86_64-linux" "aarch64-linux" ];

      shell = { mkShell, kdePackages, fenix, bacon, pkg-config, reuse, dbus, sqlx-cli, system, ... }:
        mkShell {
          QML_IMPORT_PATH =
            lib.fp.pipe
              [
                (map (pkg: "${pkg}/lib/qt-6/qml"))
                (builtins.concatStringsSep ":")
              ]
              [
                (config.inputs.quickshell.result.packages.${system}.default.override { gitRev=pins.quickshell.revision; })
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
    shells.testing = {
      systems = [ "x86_64-linux" "aarch64-linux" ];

      shell = { mkShell, libnotify, sqlitebrowser, system, ... }:
        mkShell {
          buildInputs = [ libnotify ];
          packages = [
            libnotify
            sqlitebrowser
            config.packages.default.result.${system}
            (config.inputs.quickshell.result.packages.${system}.default.override { gitRev=pins.quickshell.revision; })
          ];
        };
    };
  };
})
