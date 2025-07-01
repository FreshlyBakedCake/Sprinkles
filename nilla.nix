let
  pins = import ./npins;

  nilla = import pins.nilla;
in
nilla.create ({ config }: {
  config = {
    inputs = {
      fenix = {
        src = pins.fenix;
      };

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

      shell = { mkShell, fenix, bacon, pkg-config, reuse, dbus, sqlx-cli, ... }:
        mkShell {
          buildInputs = [ dbus ];
          packages = [
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
    shells.libnotify = {
      systems = [ "x86_64-linux" "aarch64-linux" ];

      shell = { mkShell, libnotify, ... }:
        mkShell {
          buildInputs = [ libnotify ];
          packages = [
            libnotify
          ];
        };
    };
  };
})
