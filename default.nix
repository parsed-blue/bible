{
  rustPlatform,
  glib,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  name = "kjv";
  src = ./.;
  buildInputs = [ glib ];
  nativeBuildInputs = [ pkg-config ];
  cargoLock.lockFile = ./Cargo.lock;
}
