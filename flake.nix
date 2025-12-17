{
  description = "A basic Rust project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }: let
    systems = flake-utils.lib.defaultSystems ++ ["riscv64-linux"];
  in
    flake-utils.lib.eachSystem systems (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustVersion = pkgs.rust-bin.stable.latest.default;
        packageMetadata = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
        pname = packageMetadata.name;
        inherit (packageMetadata) version;
        nodeDependencies = pkgs.buildNpmPackage {
          inherit version;
          pname = "${pname}-node-deps";
          src = ./.;
          npmDepsHash = "sha256-78WzgrRJ7lSQSFb1NDTpNsz7sPsjh+M8qxxwoSz9fXc=";
          installPhase = ''
            runHook preInstall
            mkdir -p $out/dist
            cp -r dist/* $out/dist
            cp -r node_modules $out
            runHook postInstall
          '';
        };
        rustPackageOptions = {
          inherit pname version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          BAYBRIDGE_DIST_PATH = "${nodeDependencies}/dist";
          BAYBRIDGE_CHARTJS_DIST_PATH = "${nodeDependencies}/node_modules/chart.js/dist";

          buildInputs = [
            nodeDependencies
          ];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustVersion
            pkgs.cargo-watch
            pkgs.just
            pkgs.nodejs
            pkgs.rust-analyzer
            pkgs.pkg-config
          ];
        };

        packages = {
          inherit nodeDependencies;

          default = pkgs.rustPlatform.buildRustPackage rustPackageOptions;
          static = pkgs.pkgsStatic.rustPlatform.buildRustPackage rustPackageOptions;

          docker = pkgs.dockerTools.buildLayeredImage {
            name = pname;
            tag = version;
            extraCommands = ''
              mkdir -p -m 1777 .local
            '';
            config = {
              Entrypoint = ["${self.packages.${system}.default}/bin/${pname}"];
              Cmd = ["serve"];
              User = "100";
            };
          };
        };
      }
    );
}
