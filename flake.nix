{
  description = "A tool to control hyprsunset";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        sundial = pkgs.rustPlatform.buildRustPackage {
          pname = "sundial";
          version = "1.0.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          meta = with pkgs.lib; {
            description = "A tool to control hyprsunset based on sunrise/sunset times";
            homepage = "https://github.com/tcione/sundial";
            license = licenses.mit;
            maintainers = [ "tcione" ];
          };
        };
      in
      {
        packages = {
          default = sundial;
          sundial = sundial;
        };
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            rustfmt
            rustycli
            cargo
          ];

          shellHook = ''
            echo "🏗️  SUNDIAL!"
            echo ""
          '';
        };
      });
}
