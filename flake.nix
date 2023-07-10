{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { self
    , nixpkgs
    , rust-overlay
    }:

    let
      overlays = [
        rust-overlay.overlays.default
        (self: super: {
          rustToolchain = super.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        })
      ];
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachsupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs { inherit overlays system; };
      });
    in
    {
      devShells = forEachsupportedSystem ({ pkgs }: {
        default =
          let
            xFunc = cmd: pkgs.writeScriptBin "x-${cmd}" ''
              cargo watch -x ${cmd}
            '';

            ci = pkgs.writeScriptBin "ci" ''
              cargo fmt --check
              cargo clippy
              cargo build --release
              cargo test
            '';

            scripts = [
              ci
              (builtins.map (cmd: xFunc cmd) [ "build" "check" "clippy" "run" "test" ])
            ];

            macosPkgs = pkgs.lib.optionals pkgs.stdenv.isDarwin
              (with pkgs.darwin.apple_sdk.frameworks; [ CoreServices ]);
          in
          pkgs.mkShell {
            packages = (with pkgs; [
              rustToolchain
              cargo-edit
              cargo-watch
              rust-analyzer
            ]) ++ scripts ++ macosPkgs;
          };
      });
    };
}
