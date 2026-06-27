{
  description = "MPRIS music scrobbler daemon";

  inputs = {
    # For packages we pull.
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    # So we don't have to manually definite things for each os/arch combination.
    flake-utils.url = "github:numtide/flake-utils";
    # To cache rust crates.
    naersk = {
      url = "github:nix-community/naersk";
      # Use the same nixpkgs that we've defined above.
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        naerskLib = pkgs.callPackage naersk {};
        systemDeps = with pkgs; [openssl_3 dbus];
      in {
        packages.default = naerskLib.buildPackage {
          src = ./.;
          buildInputs = systemDeps;
          nativeBuildInputs = [pkgs.pkg-config];
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer
            ]
            ++ systemDeps;

          # Used for finding system dependencies.
          nativeBuildInputs = [pkgs.pkg-config];

          # Needed so programs like rust-analyzer can find the rust source code.
          env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      }
    );
}
