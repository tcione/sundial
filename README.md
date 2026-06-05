# Sundial

A companion program for [hyprsunset](https://github.com/hyprwm/hyprsunset). It focuses on setting up screen temperature and gamma based on local sunrise and sunset times.

## Features
- Dynamic day and night temperature+gamma based on arbitrary latitude and longitude.
- Automatic hyprsunset management via hyprctl and systemctl.
- Smooth transition between day and night settings.
- Daily caching of sunset and sunrise times to avoid unecessary API calls

## Setup

### Configuration
A default configuration is generated the first time sundial runs in `.config/sundial/config.toml`. This is how it looks like:

```toml
[location]
mode = "auto"
latitude = "52.56"
longitude = "13.39"

[screen]
day_temperature = "6000"
day_gamma = "100"
night_temperature = "2800"
night_gamma = "80"
fade_duration_in_minutes = 60

[cache]
enabled = true
```

Tweak it to your liking. `location.mode` is `"auto"` (resolve coordinates via IP geolocation) or `"fixed"` (use the `latitude`/`longitude` below).

#### Declarative configuration (Home Manager)
The Home Manager module exposes a `settings` option that is written to `~/.config/sundial/config.toml`, so the whole configuration can be managed declaratively:

```nix
services.sundial = {
  enable = true;
  settings = {
    location = {
      mode = "fixed";
      latitude = "52.56";
      longitude = "13.39";
    };
    screen = {
      day_temperature = "6000";
      day_gamma = "100";
      night_temperature = "2800";
      night_gamma = "80";
      fade_duration_in_minutes = 60;
    };
    cache.enabled = true;
  };
};
```

`settings` mirrors the TOML schema above one-to-one. When set, provide a complete config (all of `location`, `screen` and `cache`) since the app rejects a partial file. When omitted, no file is written and sundial generates its own defaults on first run.

The NixOS module manages only the service and timer; declarative config is Home Manager only, since the file lives in the user's home.

### Running the program
Although this can be run as a standalone program, this is designed to be triggered in a schedule. My personal recommendation is using a oneshot systemd service + a systemd timer (this comes out of the box if you are using the nix flake).

## Recognition // Gratitude
- [Hyprland](https://github.com/hyprwm/Hyprland) and [hyprsunset](https://github.com/hyprwm/hyprsunset), since I enjoy both so much
- [SunriseSunset.io](https://sunrisesunset.io/api/) for providing the amazing API that makes this project possible
