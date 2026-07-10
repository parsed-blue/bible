{
  rustPlatform,
  glib,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  name = "parsed_blue_bible";
  src = ./.;
  buildInputs = [ glib ];
  nativeBuildInputs = [ pkg-config ];
  cargoLock.lockFile = ./Cargo.lock;
}
