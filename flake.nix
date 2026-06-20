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
            fenixPkgs.targets."x86_64-unknown-linux-musl".stable.rust-std
          ];

          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          # linux/macos native nix
          commonArgs = {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;

            buildInputs = [];
            nativeBuildInputs = [];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          multiauth14 = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });

          # linux musl static build
          musl = pkgs.pkgsCross.musl64;

          muslArgs = commonArgs // {
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
            CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${musl.stdenv.cc}/bin/x86_64-unknown-linux-musl-gcc";

            TARGET_CC = "${musl.stdenv.cc}/bin/x86_64-unknown-linux-musl-gcc";
            CC_x86_64_unknown_linux_musl = "${musl.stdenv.cc}/bin/x86_64-unknown-linux-musl-gcc";
            CXX_x86_64_unknown_linux_musl = "${musl.stdenv.cc}/bin/x86_64-unknown-linux-musl-g++";

            depsBuildBuild = (commonArgs.depsBuildBuild or []) ++ [
              musl.stdenv.cc
            ];
          };

          multiauth14-musl = craneLib.buildPackage (muslArgs // {
            cargoArtifacts = craneLib.buildDepsOnly muslArgs;
          });

          # windows cross compilation
          mingw = pkgs.pkgsCross.mingwW64;

          windowsArgs = commonArgs // {
            CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${mingw.stdenv.cc}/bin/x86_64-w64-mingw32-gcc";
			CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L native=${mingw.windows.pthreads}/lib";

			TARGET_CC = "${mingw.stdenv.cc}/bin/x86_64-w64-mingw32-gcc";
            CC_x86_64_pc_windows_gnu = "${mingw.stdenv.cc}/bin/x86_64-w64-mingw32-gcc";
            CXX_x86_64_pc_windows_gnu = "${mingw.stdenv.cc}/bin/x86_64-w64-mingw32-g++";

            depsBuildBuild = (commonArgs.depsBuildBuild or []) ++ [
              mingw.stdenv.cc
            ];

            buildInputs = (commonArgs.buildInputs or []) ++ [
              mingw.windows.pthreads
            ];

            # skip check due to cross compilation
            doCheck = false;
          };

          multiauth14-windows = craneLib.buildPackage (windowsArgs // {
            cargoArtifacts = craneLib.buildDepsOnly windowsArgs;
          });
        in {
          default = multiauth14;
          inherit multiauth14 multiauth14-musl multiauth14-windows;
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
            fenixPkgs.targets."x86_64-unknown-linux-musl".stable.rust-std
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