{
  description = "The core crate for Drone, an Embedded Operating System";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-22.05";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, utils, nixpkgs, fenix }:
    utils.lib.eachDefaultSystem (system:
      let
        rustChannel = {
          channel = "nightly";
          date = "2022-09-18";
          sha256 = "eYFYpSF2PBUJVzZGZrdtDMpVfHkypzTMLWotdEVq7eM=";
        };

        pkgs = nixpkgs.legacyPackages.${system};
        rustToolchain = with fenix.packages.${system}; combine
          (with toolchainOf rustChannel; [
            rustc
            cargo
            clippy
            rustfmt
            rust-src
          ]);
        rustAnalyzer = fenix.packages.${system}.rust-analyzer;

        cargoRdme = (
          pkgs.rustPlatform.buildRustPackage rec {
            name = "cargo-rdme";
            src = pkgs.fetchFromGitHub {
              owner = "orium";
              repo = name;
              rev = "v0.7.2";
              sha256 = "sha256-jMFBdfSd3hz3YdI1TZjJFJGzcSIrry+4zgUgV51MlZ4=";
            };
            cargoSha256 = "sha256-2swM8GLyYDyrSXzaKNbG4u1//X35Oa4SCKPqiMVhwxY=";
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl ];
            doCheck = false;
          });

        checkAll = pkgs.writeShellScriptBin "check-all" ''
          set -ex
          cargo rdme --check
          cargo fmt --all --check
          cargo clippy --workspace -- --deny warnings
          cargo clippy --workspace --features atomics -- --deny warnings
          cargo test --workspace --exclude drone-core
          cargo test --package drone-core --features std
          cargo test --package drone-core --features std,atomics
          RUSTFLAGS='--cfg loom' cargo test --package drone-core --features std --release loom
          RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace
        '';

        updateVersions = pkgs.writeShellScriptBin "update-versions" ''
          sed -i "s/\(api\.drone-os\.com\/drone-core\/\)[0-9]\+\(\.[0-9]\+\)\+/\1$(echo $1 | sed 's/\(.*\)\.[0-9]\+/\1/')/" \
            Cargo.toml ctypes/Cargo.toml macros/Cargo.toml macros-core/Cargo.toml src/lib.rs
          sed -i "/\[.*\]/h;/version = \".*\"/{x;s/\[package\]/version = \"$1\"/;t;x}" \
            Cargo.toml ctypes/Cargo.toml macros/Cargo.toml macros-core/Cargo.toml
          sed -i "/\[.*\]/h;/version = \"=.*\"/{x;s/\[.*drone-.*\]/version = \"=$1\"/;t;x}" \
            Cargo.toml macros/Cargo.toml
          sed -i "/\[.*\]/h;/version = \".*\"/{x;s/\[.*drone-config\]/version = \"$2\"/;t;x}" \
            macros/Cargo.toml
          sed -i "s/\(drone-core.*\)version = \"[^\"]\+\"/\1version = \"$1\"/" \
            src/lib.rs
        '';

        publishCrates = pkgs.writeShellScriptBin "publish-crates" ''
          cd ctypes && cargo publish
          cd macros-core && cargo publish
          sleep 30
          cd macros && cargo publish
          sleep 30
          cargo publish
        '';

        publishDocs = pkgs.writeShellScriptBin "publish-docs" ''
          dir=$(sed -n 's/.*api\.drone-os\.com\/\(.*\/.*\)\/.*\/"/\1/;T;p' Cargo.toml) \
            && rm -rf ../drone-api/$dir \
            && cp -rT target/doc ../drone-api/$dir \
            && echo '<!DOCTYPE html><meta http-equiv="refresh" content="0; URL=./drone_core">' > ../drone-api/$dir/index.html \
            && cd ../drone-api && git add $dir && git commit -m "Docs for $dir"
        '';

        shell = pkgs.mkShell {
          name = "native";
          nativeBuildInputs = [
            rustToolchain
            rustAnalyzer
            cargoRdme
            checkAll
            updateVersions
            publishCrates
            publishDocs
          ];
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      in
      {
        devShells = {
          native = shell;
          default = shell;
        };
      }
    );
}
