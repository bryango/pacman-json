{
  lib,
  rustPlatform,
  pkg-config,
  mold-wrapped,
  pacman,
  installShellFiles,
}:

rustPlatform.buildRustPackage {
  name = "pacjump";
  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
    installShellFiles
    mold-wrapped # fast linker, configured below
  ];

  env =
    let
      rustflagMold = "-Clink-arg=-fuse-ld=mold";
    in
    {
      RUSTFLAGS = rustflagMold;
      RUSTDOCFLAGS = rustflagMold;
    };

  buildInputs = [
    pacman
  ];

  cargoTestFlags = [
    "--lib"
    "--bins" # skip integration tests (slow & needs pacman database)
    "--no-fail-fast" # make post-mortem easier
  ];

  postInstall = ''
    pushd completions
    installShellCompletion --cmd hydra-check \
      --bash pacjump.bash \
      --fish pacjump.fish \
      --zsh _pacjump
    popd
  '';

  meta = {
    description = "Dump pacman packages information in JSON";
    homepage = "https://github.com/bryango/pacman-json";
    license = lib.licenses.gpl3Only;
    maintainers = with lib.maintainers; [ bryango ];
    mainProgram = "pacjump";
  };
}
