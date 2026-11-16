# Eye Saver for Linux

A native Linux remake of [Eye Saver](https://www.leosoft.us/) — an open-source-style break reminder that helps you avoid eye strain and sedentary work habits.

Built with **Electron** and packaged as **AppImage** and **.deb** via **GitHub Actions** on Ubuntu.

## Features

- **Break reminders** — fullscreen break popups with configurable interval and duration; skip or delay (1 / 5 minutes)
- **Micro pauses** — short, frequent rests with optional sound notification
- **Blue light filter** — warm color overlay with adjustable intensity
- **Screen effects** — grayscale, invert, and custom color overlay
- **Health reminders** — stretch and water notifications
- **Idle detection** — timers reset after periods of inactivity (`xprintidle`)
- **System tray** — quick toggles for breaks, micro pauses, and blue filter
- **Prevent sleep** — keep the display awake during work sessions
- **Autostart** — launch automatically at login
- **Single instance** — only one copy runs at a time
- **Persistent settings** — stored in `~/.config/eye-saver-linux/settings.json`

## Install

### Download a release

Grab the latest **AppImage** or **.deb** from the [Releases](../../releases) page (or the Actions artifacts on any successful build).

**AppImage:**

```bash
chmod +x "Eye Saver-2.48.0.AppImage"
./Eye\ Saver-2.48.0.AppImage
```

**Debian / Ubuntu / Linux Mint:**

```bash
sudo dpkg -i eye-saver-linux_2.48.0_amd64.deb
```

### Build from source

Requires [Node.js](https://nodejs.org/) 20+.

```bash
git clone https://github.com/shuvo582/Eye-Saver-Linux.git
cd Eye-Saver-Linux/linux-app
npm ci
npm start          # run in development
npm run dist:all   # package AppImage + deb into dist/
```

## Development

| Command | Description |
|---------|-------------|
| `npm start` | Launch the app in development mode |
| `npm run pack` | Package an unpacked build only |
| `npm run dist:AppImage` | Build AppImage |
| `npm run dist:deb` | Build .deb |
| `npm run dist:all` | Build both targets |

### Project layout

```
.github/workflows/build.yml   # CI: builds AppImage + deb on ubuntu-latest
linux-app/
  src/main.js                 # Electron main process (timers, tray, overlays, IPC)
  src/preload.js              # contextBridge API
  src/index.html              # settings UI
  src/break.html              # fullscreen break / micro-pause overlay
  src/renderer.js             # settings UI logic
  src/styles.css
```

### Continuous integration

Every push to `main` triggers [`.github/workflows/build.yml`](.github/workflows/build.yml):

1. Checkout → Node 22 → `npm ci`
2. `npm run dist:all` (AppImage + deb)
3. Upload artifacts (kept 30 days)

Pushing a tag matching `v*` also attaches the binaries to a **GitHub Release**.

## Screenshots

<!-- Optional: add screenshots of the settings window and break overlay here -->

## License

MIT — see [LICENSE](LICENSE).

---

**Note:** This is an independent Linux recreation for personal use. The original Eye Saver is a commercial product by Leosoft Ltd.; this project is not affiliated with or endorsed by Leosoft.
