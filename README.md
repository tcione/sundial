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

Tweak it to your liking.

### Running the program
Although this can be run as a standalone program, this is designed to be triggered in a schedule. My personal recommendation is using a oneshot systemd service + a systemd timer (this comes out of the box if you are using the nix flake).

## Roadmap // TODO
- [x] Nix flake for easy setup
- [ ] Maybe automatically fetch user coordinates

## Notes
- This is the project I'm using to learn Rust properly, so you'll probably find some weird patterns/ideas
- I've used Claude Code as a learning tool for the project. I've documented my experience in 80aa98f7a767355fee2088f0d67ece188d350e61

## Recognition // Gratitude
- [Hyprland](https://github.com/hyprwm/Hyprland) and [hyprsunset](https://github.com/hyprwm/hyprsunset), since I enjoy both so much
- [SunriseSunset.io](https://sunrisesunset.io/api/) for providing the amazing API that makes this project possible
