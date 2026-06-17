{
  description = "multiauth14";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, fenix }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          fenixPkgs = fenix.packages.${system};

          rustToolchain = fenixPkgs.combine [
            fenixPkgs.stable.cargo
            fenixPkgs.stable.rustc
            fenixPkgs.stable.rust-src
            fenixPkgs.stable.rustfmt
            fenixPkgs.stable.clippy
            fenixPkgs.stable.rust-analyzer
            fenixPkgs.targets."x86_64-pc-windows-gnu".stable.rust-std
          ];

          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          # linux/macos
          commonArgs = {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;

            buildInputs = [];
            nativeBuildInputs = [];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          multiauth14 = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });

          # windows cross compilation
          mingw = pkgs.pkgsCross.mingwW64;

          windowsArgs = commonArgs // {
            CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${mingw.stdenv.cc}/bin/x86_64-w64-mingw32-gcc";
          };

          multiauth14-windows = craneLib.buildPackage (windowsArgs // {
            cargoArtifacts = craneLib.buildDepsOnly windowsArgs;
          });
        in {
          default = multiauth14;
          inherit multiauth14 multiauth14-windows;
        }
      );

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          fenixPkgs = fenix.packages.${system};

          rustToolchain = fenixPkgs.combine [
            fenixPkgs.stable.cargo
            fenixPkgs.stable.rustc
            fenixPkgs.stable.rust-src
            fenixPkgs.stable.rustfmt
            fenixPkgs.stable.clippy
            fenixPkgs.stable.rust-analyzer
            fenixPkgs.targets."x86_64-pc-windows-gnu".stable.rust-std
          ];

          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        in {
          default = craneLib.devShell {
            inputsFrom = [ self.packages.${system}.multiauth14 ];

            packages = [
              rustToolchain
              pkgs.cargo-watch
            ];
          };
        }
      );
    };
}