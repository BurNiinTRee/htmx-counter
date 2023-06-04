{
  inputs = {
    devenv.url = "github:cachix/devenv";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = inputs @ {
    flake-parts,
    fenix,
    bntr,
    devenv,
    nixpkgs,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} ({...}: {
      systems = ["x86_64-linux"];

      imports = [bntr.flakeModules.nixpkgs devenv.flakeModule];

      perSystem = {
        nixpkgs.overlays = [fenix.overlays.default];
        devenv.shells.default = {pkgs, ...}: {
          languages.c.enable = true;
          packages = [
            pkgs.cargo
            pkgs.cargo-watch
            pkgs.clippy
            pkgs.rustfmt
            pkgs.rustc
            # pkgs.fenix.default.toolchain
            pkgs.rust-analyzer
            pkgs.sqlite
            pkgs.sqlx-cli
            pkgs.mold
          ];
          env = {
            DATABASE_URL = "sqlite:data.db?mode=rwc";
            CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = ["-Clink-arg=-fuse-ld=mold" "-Clinker=clang"];
          };
        };
      };
    });
}
