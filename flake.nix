{
  description = "A website serving the the King James Version of the Bible";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    {
      self,
      nixpkgs,
      naersk,
      systems,
    }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          naerskLib = pkgs.callPackage naersk { };
        in
        {
          default = naerskLib.buildPackage {
            src = ./.;
          };
        }
      );
      devShells = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          naerskLib = pkgs.callPackage naersk { };
        in
        {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            glib
            nixfmt-rfc-style
          ];
          # nativeBuildInputs = [ pkgs.pkg-config ];
          env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        }
      );
    };
}
