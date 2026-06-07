# Sundial

A companion program for [hyprsunset](https://github.com/hyprwm/hyprsunset). It focuses on setting up screen temperature and gamma based on local sunrise and sunset times.

## Features
- Dynamic day and night temperature+gamma based on fixed or automatically detected latitude and longitude.
- Automatic hyprsunset management via hyprctl and systemctl.
- Smooth transition between day and night settings.
- Daily caching of location to avoid repeated detection

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

Tweak it to your liking. `location.mode` is `"auto"` (detect coordinates automatically — see below) or `"fixed"` (use the `latitude`/`longitude` values directly).

#### Auto location detection

In `auto` mode sundial tries the following sources in order, using the first one that succeeds:

1. **GeoClue2** — queries the system location daemon via D-Bus. VPN-independent; uses WiFi access point scanning, GPS, or other hardware sources depending on what your system provides.
2. **System timezone** — reads the IANA timezone from `/etc/localtime` and maps it to representative city coordinates. Fully offline, no network call required.
3. **Cached location** — the last successfully detected position.
4. **Config default** — the `latitude`/`longitude` values in `config.toml`.

GeoClue2 requires the app to be explicitly authorized. How to do that depends on your setup:

**NixOS** — add to your system configuration:

```nix
services.geoclue2 = {
  enable = true;
  appConfig."sundial" = {
    isAllowed = true;
    isSystem = false;
    users = [];
  };
};
```

**GNOME / KDE** — the desktop location agent authorizes apps on your behalf automatically. No extra steps needed as long as location services are enabled in system settings.

**Other distros (manual)** — add the following to `/etc/geoclue/geoclue.conf`:

```ini
[sundial]
allowed=true
system=false
users=
```

Then restart the GeoClue2 service: `sudo systemctl restart geoclue`

If GeoClue2 is unavailable or not authorized, sundial falls back silently to the timezone-based approximation.

#### Location cache

Because sundial is designed to run on short intervals, re-running the full detection chain on every invocation would be wasteful. Cache is refreshed every system reboot OR every 24h (whatever happens first).

Cache is stored in the user data directory (typically `~/.local/share/sundial/location.json`). It can be disabled by setting `cache.enabled = false` in `config.toml`.

#### Declarative configuration (Home Manager)

The Home Manager module exposes a "settings" attributes:

```nix
services.sundial = {
  enable = true;
  interval = "*:0/1"; # optional
  logLevel = "info"; # optional
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

`settings` mirrors the TOML schema above one-to-one and all keys are mandatory.

The NixOS module manages only the service and timer; declarative config is Home Manager only, since the file lives in the user's home.

### Running the program
Although this can be run as a standalone program, this is designed to be triggered in a schedule every 1 to 10 mins. My personal recommendation is using a oneshot systemd service + a systemd timer (this comes out of the box if you are using the nix flake).
