{
  inputs = {
    flake-compat.url = "github:edolstra/flake-compat/refs/pull/65/head";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-compat,
    }:
    let
      overlays = [
        (import rust-overlay)
        self.overlays.default
      ];
      pkgsFor = system: import nixpkgs { inherit system overlays; };

      targetSystems = [
        "aarch64-linux"
        "x86_64-linux"
      ];
      mkRustToolchain = pkgs: pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      overlays.default = final: prev: { inherit (self.packages.${prev.system}) ewwii ewwii-wayland; };

      packages = nixpkgs.lib.genAttrs targetSystems (
        system:
        let
          pkgs = pkgsFor system;
          rust = mkRustToolchain pkgs;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
          version = (builtins.fromTOML (builtins.readFile ./crates/ewwii/Cargo.toml)).package.version;
        in
        rec {
          ewwii = rustPlatform.buildRustPackage {
            version = "${version}-dirty";
            pname = "ewwii";

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [
              "--bin"
              "ewwii"
            ];

            nativeBuildInputs = with pkgs; [
              pkg-config
              wrapGAppsHook
            ];
            buildInputs = with pkgs; [
              gtk3
              librsvg
              gtk-layer-shell
              libdbusmenu-gtk3
            ];
          };

          ewwii-wayland = nixpkgs.lib.warn "`ewwii-wayland` is deprecated due to ewwii building with both X11 and wayland support by default. Use `ewwii` instead." ewwii;
          default = ewwii;
        }
      );

      devShells = nixpkgs.lib.genAttrs targetSystems (
        system:
        let
          pkgs = pkgsFor system;
          rust = mkRustToolchain pkgs;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.ewwii ];
            packages = with pkgs; [
              deno
              mdbook
              zbus-xmlgen
            ];

            RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
          };
        }
      );

      formatter = nixpkgs.lib.genAttrs targetSystems (system: (pkgsFor system).nixfmt-rfc-style);
    };
}
