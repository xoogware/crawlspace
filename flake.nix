# Copyright (c) 2024 Andrew Brower.
# This file is part of Crawlspace.
#
# Crawlspace is free software: you can redistribute it and/or
# modify it under the terms of the GNU Affero General Public
# License as published by the Free Software Foundation, either
# version 3 of the License, or (at your option) any later version.
#
# Crawlspace is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
# Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public
# License along with Crawlspace. If not, see
# <https://www.gnu.org/licenses/>.
{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {
    flake-parts,
    rust-overlay,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        version = "0.0.1";
        rust-bin = pkgs.pkgsBuildHost.rust-bin;
        rustToolchain = rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust-bin.selectLatestNightlyWith (toolchain: rustToolchain);
          rustc = rust-bin.selectLatestNightlyWith (toolchain: rustToolchain);
          clippy = rust-bin.selectLatestNightlyWith (toolchain: rustToolchain);
        };
      in {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [
            self'.packages.default
          ];

          packages = with pkgs; [
            rust-analyzer
          ];
        };

        packages = let
          mkCrawlspace = buildType:
            rustPlatform.buildRustPackage {
              pname = "crawlspace";
              inherit version;
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
              cargoLock.outputHashes."fastnbt-2.5.0" = "E4WI6SZgkjqUOtbfXfKGfpFH7btEh5V0KpMXSIsuh08=";
              inherit buildType;
              dontStrip = buildType == "debug";
              buildInputs = with pkgs; [pkg-config openssl];
            };
        in {
          default = mkCrawlspace "debug";
          release = mkCrawlspace "release";
        };
      };
    };
}
