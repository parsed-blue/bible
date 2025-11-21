{
  description = "A website serving the the World English People Version of the Bible";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      naersk,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) { inherit system; };
        naersk' = pkgs.callPackage naersk {};
      in
      rec {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [rustc cargo rustfmt rust-analyzer nixfmt-rfc-style];
        };
      }
    );
}
