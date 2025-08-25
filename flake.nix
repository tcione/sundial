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
            maintainers = [ "@tcione" ];
          };
        };

        nixosModule = { config, lib, pkgs, ... }: with lib; {
          options.services.sundial = {
            enable = mkEnableOption "sundial service";

            package = mkOption {
              type = types.package;
              default = sundial;
              description = "The sundial package to use";
            };

            interval = mkOption {
              type = types.str;
              default = "*:0/5";
              description = "How often to run sundial (systemd format)";
            };

            logLevel = mkOption {
              type = types.str;
              default = "info";
              description = "Application log level. Options: error, warn, info, debug, trace";
            };
          };

          config = mkIf config.services.sundial.enable {
            systemd.user.services.sundial = {
              Unit.Description = "sets screen temperature based on sunrise/sunset times";
              Service = {
                Type = "oneshot";
                ExecStart = "${config.services.sundial.package}/bin/sundial";
                Environment = "RUST_LOG=${config.services.sundial.logLevel}";
              };
              Install.WantedBy = [ "hyprland-session.target" ];
            };

            systemd.user.timers.sundial = {
              Unit.Description = "timer for sundial service";
              Timer = {
                Unit = "sundial.service";
                OnCalendar = config.services.sundial.interval;
                OnBootSec = "1m";
              };
              Install.WantedBy = [ "timers.target" ];
            };
          };
        };

        homeManagerModule = nixosModule;
      in
      {
        packages = {
          default = sundial;
          sundial = sundial;
        };

        nixosModules.default = nixosModule;
        homeManagerModules.default = homeManagerModule;

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
