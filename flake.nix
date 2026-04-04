{
  description = "A website serving the the World English People Version of the Bible";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";

  };
  outputs =
    {
      nixpkgs,
      naersk,
      flake-utils,
      fenix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) { inherit system; };
        fenix' = (import fenix { inherit system; });
        naersk' = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            nixfmt-rfc-style
            fenix'.latest.toolchain
          ];
          nativeBuildInputs = [ ];
        };
      }
    );
}
