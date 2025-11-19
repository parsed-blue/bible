{
  description = "A website serving the the King James Version of the Bible";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    systems.url = "github:nix-systems/default";
  };

  outputs = { self, nixpkgs, naersk, systems }: let
    eachSystem = nixpkgs.lib.genAttrs (import systems);
    pkgs = nixpkgs.legacyPackages."aarch64-darwin";
    naerskLib = pkgs.callPackage naersk {};
  in {
    packages = eachSystem (system: {
      default = naerskLib.buildPackage {
        src = ./.;
        buildInputs = [ pkgs.glib ];
        nativeBuildInputs = [ pkgs.pkg-config ];
      };
    });
    # packages.aarch64-darwin.default = pkgs.callPackage ./default.nix { };
    devShells.aarch64-darwin.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        cargo rustc rustfmt clippy rust-analyzer glib
      ];
      nativeBuildInputs = [pkgs.pkg-config];
      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}
