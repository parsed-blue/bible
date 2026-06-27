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
      self,
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
        commitHash = if (self ? rev) then self.rev else "dirty";
      in
      {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
          cargoBuildOptions = opts: opts ++ [ 
            "--config" "env.COMMIT_HASH=${pkgs.lib.escapeShellArg commitHash}" 
          ];
        };
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            nixfmt
            fenix'.latest.toolchain
          ];
          nativeBuildInputs = [ ];
        };
      }
    );
}
