{
  description = "A basic Rust project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustVersion = pkgs.rust-bin.stable.latest.default;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustVersion
            pkgs.cargo-watch
            pkgs.just
            pkgs.nodejs
            pkgs.rust-analyzer
            pkgs.pkg-config
            pkgs.openssl
          ];

          shellHook = ''
            echo "Rust dev environment ready!"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "baybridge";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          nativeBuildInputs = [pkgs.pkg-config pkgs.perl pkgs.cmake];

          buildInputs = [
          pkgs.pkg-config
            pkgs.openssl
          ];
        };

        packages.docker = pkgs.dockerTools.buildImage {
          name = "baybridge";
          tag = "latest";
          copyToRoot = [
            self.packages.${system}.default
          ];
          config.EntryPoint = [ "/bin/baybridge" ];
        };
      }
    );
}
