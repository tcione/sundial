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
        tomlFormat = pkgs.formats.toml { };

        sundial = pkgs.rustPlatform.buildRustPackage {
          pname = "sundial";
          version = "1.5.0";

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

        commonOptions = lib: with lib; {
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

        serviceConfig = cfg: {
          systemd.user.services.sundial = {
            Unit.Description = "sets screen temperature based on sunrise/sunset times";
            Service = {
              Type = "oneshot";
              ExecStart = "${cfg.package}/bin/sundial";
              Environment = "RUST_LOG=${cfg.logLevel}";
            };
            Install.WantedBy = [ "hyprland-session.target" ];
          };

          systemd.user.timers.sundial = {
            Unit.Description = "timer for sundial service";
            Timer = {
              Unit = "sundial.service";
              OnCalendar = cfg.interval;
              OnBootSec = "1m";
            };
            Install.WantedBy = [ "timers.target" ];
          };
        };

        nixosModule = { config, lib, pkgs, ... }: with lib; {
          options.services.sundial = commonOptions lib;

          config = mkIf config.services.sundial.enable
            (serviceConfig config.services.sundial);
        };

        homeManagerModule = { config, lib, pkgs, ... }: with lib;
          let cfg = config.services.sundial; in {
            options.services.sundial = commonOptions lib // {
              settings = mkOption {
                type = tomlFormat.type;
                default = { };
                example = literalExpression ''
                  {
                    location = { mode = "fixed"; latitude = "52.56"; longitude = "13.39"; };
                    screen = {
                      day_temperature = "6000";
                      day_gamma = "100";
                      night_temperature = "2800";
                      night_gamma = "80";
                      fade_duration_in_minutes = 60;
                    };
                    cache.enabled = true;
                  }
                '';
                description = ''
                  Configuration written to ~/.config/sundial/config.toml. Mirrors the
                  application's TOML schema (sections: location, screen, cache). If set,
                  provide a complete config (all of location, screen, cache) since the
                  app rejects a partial file. When empty, no file is written and the app
                  generates its own defaults on first run.
                '';
              };
            };

            config = mkIf cfg.enable (mkMerge [
              (serviceConfig cfg)
              {
                xdg.configFile."sundial/config.toml" = mkIf (cfg.settings != { }) {
                  source = tomlFormat.generate "sundial-config.toml" cfg.settings;
                };
              }
            ]);
          };
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
