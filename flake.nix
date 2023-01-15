{
  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { self
    , flake-utils
    , nixpkgs
    , rust-overlay
    }:

    flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [
        (import rust-overlay)
        (self: super: {
          rustToolchain = super.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        })
      ];

      pkgs = import nixpkgs { inherit system overlays; };
      inherit (pkgs) mkShell writeScriptBin;

      xFunc = cmd: writeScriptBin "x-${cmd}" ''
        cargo watch -x ${cmd}
      '';

      ci = writeScriptBin "ci" ''
        cargo fmt --check
        cargo clippy
        cargo build --release
        cargo test
      '';

      scripts = [
        ci
        (builtins.map (cmd: xFunc cmd) [ "build" "check" "run" "test" ])
      ];

      macosPkgs = pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [ CoreServices ]);
    in
    {
      devShells.default = mkShell {
        packages = (with pkgs; [
          rustToolchain
          cargo-edit
          cargo-watch
          rust-analyzer
        ]) ++ scripts ++ macosPkgs;
      };
    });
}
