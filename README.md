# Squat Souls

Turn what happens in your games into bodyweight exercises.

**[Download](#download)** for Windows, macOS and Linux.

![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-informational)
![Tauri 2](https://img.shields.io/badge/Tauri-2-informational)
![Vue 3](https://img.shields.io/badge/Vue-3-informational)
[![License: MIT](https://img.shields.io/badge/license-MIT-informational)](LICENSE)
[![Latest release](https://img.shields.io/github/v/release/Semche95/SquatSouls?label=release)](https://github.com/Semche95/SquatSouls/releases/latest)
[![Build](https://github.com/Semche95/SquatSouls/actions/workflows/build.yml/badge.svg)](https://github.com/Semche95/SquatSouls/actions/workflows/build.yml)

![Play tab with a set to do](docs/screenshots/play-set-to-do.png)

## What is it?

A small desktop app you keep open next to your game. When something happens in game (you die, beat a boss), you click that event and the app gives you a few bodyweight exercises with a random number of reps. Do them before you carry on playing.

You configure your own games, events and exercises. The defaults are only a starting point.

## Features

- Exercises come out in rounds, so each one comes out before any comes back.
- Per-session limit on each event, and jokers to halve or skip an exercise.
- Intensity of the session (Easy, Normal, Hard) to scale the numbers on tired days, without touching the configuration.
- Recovery countdown between exercises, and a timer for exercises counted in time.
- Summary of the session and of the last 7 days, plus the full history, each session copyable to paste in a messaging app.
- Keyboard shortcuts, Steam game detection, notification area icon.
- Share games and exercises with other players.
- Interface in English and French.

| Recovery | History | Settings |
| --- | --- | --- |
| ![Recovery between two exercises](docs/screenshots/recovery.png) | ![History tab](docs/screenshots/history.png) | ![Settings tab](docs/screenshots/settings.png) |

| Program: games | Program: exercises |
| --- | --- |
| ![Games of the Program tab](docs/screenshots/program-games.png) | ![Exercises of the Program tab](docs/screenshots/program-exercises.png) |

## Download

| OS | Download | Status |
| --- | --- | --- |
| Windows | [Installer (.exe)](https://github.com/Semche95/SquatSouls/releases/latest/download/SquatSouls_x64-setup.exe) | Tested |
| macOS | [Apple Silicon (.dmg)](https://github.com/Semche95/SquatSouls/releases/latest/download/SquatSouls_aarch64.dmg), [Intel (.dmg)](https://github.com/Semche95/SquatSouls/releases/latest/download/SquatSouls_x64.dmg) | Not tested |
| Linux | [AppImage](https://github.com/Semche95/SquatSouls/releases/latest/download/SquatSouls_amd64.AppImage), [.deb](https://github.com/Semche95/SquatSouls/releases/latest/download/SquatSouls_amd64.deb), [.rpm](https://github.com/Semche95/SquatSouls/releases/latest/download/SquatSouls_x86_64.rpm) | Developed and used on Arch Linux |

Requirements, those of [Tauri 2](https://v2.tauri.app/start/prerequisites/): Windows 10 or later (the installer adds WebView2 if it is missing), macOS 10.15 or later, a 64-bit Linux with WebKitGTK 4.1 (Ubuntu 22.04, Debian 12 or later).

These links always point to the latest version, older ones are on the [Releases](https://github.com/Semche95/SquatSouls/releases) page. The app then updates itself, except the `.deb` and `.rpm` packages.

> [!WARNING]
> **macOS: the app is not signed by Apple**, so macOS blocks it the first time you open it. To open it:
>
> 1. Open the app once and close the warning.
> 2. Go to **System Settings**, **Privacy & Security**.
> 3. Click **Open Anyway** next to the Squat Souls message, then confirm.

## Usage

1. In **Play**, choose your game, then click **Start the session** when you start playing.
2. You die in game: click **Death**, or press 1. A set appears in **To do**, for example 14 squats.
3. Do them, then click **Done** or press Enter. A recovery starts.
4. When you stop playing, click **End the session**. It shows up in **History**.

| Key | Action |
| --- | --- |
| 1 to 9 | Trigger the event with that number |
| Enter | Mark the next exercise done |
| H | Halve it (1 joker) |
| J | Skip it |
| T | Start its timer, if it is timed |
| Space | Hold or resume the timer |
| Backspace | Cancel the set |
| Escape | Skip the recovery or stop the timer |
| Ctrl+Q (Cmd+Q on macOS) | Quit the app |

Games, events and exercises are edited in **Program**, the options in **Settings**. The full rules are in [docs/BEHAVIOR.md](docs/BEHAVIOR.md). Where the database and logs are stored: [docs/DATA-AND-LOGS.md](docs/DATA-AND-LOGS.md).

## Linux notes

The app forces `GDK_BACKEND=x11` and `WEBKIT_DISABLE_DMABUF_RENDERER=1`, because WebKitGTK is unstable under Wayland. Set them yourself to override, for example `GDK_BACKEND=wayland squat-souls`.

## Privacy

Everything stays on your computer: no account, no telemetry. The app only goes online to look for an update on GitHub at launch, and, when Steam detection is on, to ask the Steam Store for the name of a game missing from your configuration.

## Report a problem

Open an [issue](https://github.com/Semche95/SquatSouls/issues) with your system, the app version (shown in **About**) and the steps that lead to the problem. For a crash or an error, attach the log file: its location is in [docs/DATA-AND-LOGS.md](docs/DATA-AND-LOGS.md#logs).

## Development

```bash
pnpm tauri dev             # run in development
cd src-tauri && cargo test # backend tests
pnpm test                  # interface tests
```

Project layout, translations, rules and releases: [CONTRIBUTING.md](CONTRIBUTING.md).

### Build from source

Install Rust (stable), Node.js (LTS), pnpm and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) of your system.

#### Ubuntu, Debian

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf gstreamer1.0-plugins-base gstreamer1.0-plugins-good
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

Then install [Node.js LTS](https://nodejs.org/en/download) and run `npm install -g pnpm`.

#### Arch Linux

```bash
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl appmenu-gtk-module libappindicator-gtk3 librsvg rustup nodejs pnpm patchelf gst-plugins-base gst-plugins-good
rustup default stable
```

#### Windows

In PowerShell, then open a new terminal:

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install --id Rustlang.Rustup
winget install --id OpenJS.NodeJS.LTS
npm install -g pnpm
```

#### macOS

With [Homebrew](https://brew.sh):

```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
brew install node pnpm
```

#### Build

On every system:

```bash
git clone https://github.com/Semche95/SquatSouls.git
cd SquatSouls
pnpm install
pnpm tauri build   # packages in src-tauri/target/release/bundle/
```

## License

[MIT](LICENSE)
