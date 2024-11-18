{
  description = "pacjump: dump pacman packages information in JSON";

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;
      supportedSystems = [
        "aarch64-linux"
        "i686-linux"
        "x86_64-linux"
      ];
      forEachSystem = f: lib.genAttrs supportedSystems (system: f {
        pkgs = nixpkgs.legacyPackages.${system};
        /** final packages set (of a given system) provided in this flake */
        final = self.packages.${system};
      });
    in
    {
      packages = forEachSystem ({ pkgs, final }: {
        pacjump = pkgs.callPackage ./package.nix { };
        default = final.pacjump;
      });

      devShells = forEachSystem ({ pkgs, final }: {
        default = final.pacjump.overrideAttrs ({ nativeBuildInputs, ... }: {

          nativeBuildInputs = with pkgs.buildPackages; [
            rustup # shell completions & +nightly support for doctest coverage
            cargo-tarpaulin # show test coverage

            # for integration tests
            pacman
            jq
          ] ++ nativeBuildInputs;
        });
      });
    };
}
