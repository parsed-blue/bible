{
  description = "A website serving the the King James Version of the Bible";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, naersk }: let
    pkgs = nixpkgs.legacyPackages."aarch64-darwin";
    naerskLib = pkgs.callPackage naersk {};
  in {
    # packages.aarch64-darwin.default = pkgs.callPackage ./default.nix { };
    packages.aarch64-darwin.default = naerskLib.buildPackage {
      src = ./.;
      buildInputs = [ pkgs.glib ];
      nativeBuildInputs = [ pkgs.pkg-config ];
    };
    devShells.aarch64-darwin.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        cargo rustc rustfmt clippy rust-analyzer glib
      ];
      nativeBuildInputs = [pkgs.pkg-config];
      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}
