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
    in
    {
      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          rustToolchain
          cargo-edit
          cargo-watch
          rust-analyzer
        ];
      };
    });
}
