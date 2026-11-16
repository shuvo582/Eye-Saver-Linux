# Eye Saver — Project Context & Handoff Notes

> **Purpose:** Persist full session context so a restarted OpenCode/REA session can continue without re-discovery.
> **Last updated:** 2026-11-16
> **Working directory:** `/home/shuvo/Opencode/Eye_Saver`

---

## 1. Goal

1. Reverse-engineer the Windows-only installer `Eye_Saver-setup-2.48.exe` (Inno Setup / .NET WPF).
2. Recreate it as a **native Linux app** (Electron + GitHub Actions on `ubuntu-latest`), **not** a Windows `.exe`.
3. Pipeline: **REA + OpenCode (Linux) → source → GitHub Actions → Linux artifacts** (AppImage / `.deb`).

Original plan is also summarized in `Readme.md` (Linux build target, no Windows runner).

---

## 2. System environment (as of session)

| Item | Value |
|------|--------|
| OS | Linux Mint 22.3 (`zena`), x86_64 |
| Shell | fish (`/usr/bin/fish`) |
| User | `shuvo` |
| Node (via fnm) | **v24.22.3** (required by REA; system Node 23 was unsupported) |
| fnm | `~/.local/share/fnm` (installed this session) |
| Stable node symlink | `~/.local/bin/node` → `~/.local/share/fnm/node-versions/v24.22.3/installation/bin/node` |
| npm | 11.19.1 (under Node 24) |
| gh CLI | `~/.local/bin/gh` (v2.63.2, **not yet authenticated**) |
| JDK | Temurin **25.0.5** at `~/tools/jdk25` (Ghidra requires JDK 25+) |
| Ghidra | **12.2** at `~/tools/ghidra_12.2_PUBLIC` |
| REA source clone | `~/tools/rea` (depth-1 clone of morluto/rea) |
| Disk | ~24G free at session start (after Ghidra ~543MB + JDK) |

**Shell profile additions (already written):**
- `~/.bashrc` and `~/.config/fish/config.fish`: `JAVA_HOME`, `GHIDRA_INSTALL_DIR`, `PATH` for jdk25/bin
- fnm fish conf: `~/.config/fish/conf.d/fnm.fish`

**When starting a new bash session for REA commands:**
```bash
export FNM_DIR="$HOME/.local/share/fnm"
export PATH="$HOME/tools/jdk25/bin:$FNM_DIR:$HOME/.local/bin:$PATH"
eval "$(fnm env --shell bash)"   # or open fish (fnm auto-loads)
export JAVA_HOME="$HOME/tools/jdk25"
export GHIDRA_INSTALL_DIR="$HOME/tools/ghidra_12.2_PUBLIC"
node -v   # expect v24.22.3
```

---

## 3. REA installation & OpenCode wiring

### What was done
1. **Node 24** installed via fnm; default set to 24; stable `~/.local/bin/node` symlink so MCP survives shell restarts (avoids temporary `fnm_multishells` path).
2. **REA agent package** acquired via `npx -y rea-agents@latest` (version at session: **6.9.0**).
3. **Setup applied** for OpenCode:
   ```bash
   npx -y rea-agents@latest setup --client opencode --yes --json
   ```
4. **Config written:** `~/.config/opencode/opencode.jsonc`
   - Backup: `~/.config/opencode/opencode.jsonc.rea.backup`
   - Context7 MCP preserved; `rea` MCP added.
5. **Skill installed:** `~/.agents/skills/reverse-engineer-anything/` (SKILL.md + references/)
6. **Ghidra + JDK** configured for REA (see §4).

### Current REA MCP entry (in opencode.jsonc)
```json
"rea": {
  "type": "local",
  "command": [
    "/home/shuvo/.local/bin/node",
    "/home/shuvo/.npm/_npx/af752b90495482d7/node_modules/.bin/rea",
    "mcp"
  ],
  "enabled": true,
  "env": {
    "GHIDRA_INSTALL_DIR": "/home/shuvo/tools/ghidra_12.2_PUBLIC",
    "JAVA_HOME": "/home/shuvo/tools/jdk25",
    "PATH": "<jdk25/bin>:+user bins:+system paths>"
  }
}
```

> **Note:** The npx cache path `af752b90495482d7` is pinned by REA setup. If npm cache is cleared, re-run:
> `npx -y rea-agents@latest setup --client opencode` and re-approve.

### Verification (last run)
```bash
npx -y rea-agents@latest doctor --client opencode --provider ghidra --json
```
Result:
- `healthy: true`
- OpenCode registration: **aligned**
- Skill: **aligned**, version **62**
- Ghidra: **available**, providerVersion **12.2**

**After restarting OpenCode:** ask the agent to list REA tools / call `binary_session` to confirm MCP is live.

### Update / uninstall
```bash
npx rea-agents@latest setup --client opencode   # refresh after REA updates
npx rea-agents@latest doctor --client opencode --json
# Global CLI optional:
# npm install --global rea-agents && rea update
```

---

## 4. Ghidra setup (REA native provider)

| Item | Path / value |
|------|----------------|
| Ghidra install | `~/tools/ghidra_12.2_PUBLIC` |
| Official zip source | GitHub `NationalSecurityAgency/ghidra` release `Ghidra_12.2_build` / `ghidra_12.2_PUBLIC_20261016.zip` (~544MB) |
| JDK 25 | `~/tools/jdk25` → `~/tools/jdks/jdk-25.0.5+8` (Temurin, user-space; system only had JDK 21) |
| Headless | `~/tools/ghidra_12.2_PUBLIC/support/analyzeHeadless` |

Doctor previously failed on `java: runtime only JDK 21` until JDK 25 was installed and `JAVA_HOME`/`PATH` exported into the REA MCP `env` map.

---

## 5. Target analysis (REA findings)

### Input artifact
| Field | Value |
|-------|--------|
| Path | `/home/shuvo/Opencode/Eye_Saver/Eye_Saver-setup-2.48.exe` |
| Size | 3,553,728 bytes (~3.4 MB) |
| SHA256 | `30b56b9aa2cabde21a9b4a898d4299a7bbca6235fb5c0ee9939d938cdb20e6b6` |
| Format | PE32 GUI i386 |
| Installer | **Inno Setup** 5.5.3 / setup data **5.6.0 (unicode)** |
| Product | **Eye Saver** — Company: **Leosoft ltd.**, version **2.48** |

### Extraction
Used **innoextract 1.9** (user-space: `/tmp/innoextract-1.9-linux/bin/amd64/innoextract`):
```bash
innoextract -d /tmp/eyesaver_inno Eye_Saver-setup-2.48.exe
```
Extracted files:
| Path | Notes |
|------|--------|
| `app/Eye Saver.exe` | ~2.6 MB — **PE32 Mono/.NET assembly**, WPF (.NETFramework **v4.5**) |
| `app/updater.exe` | .NET console updater |
| `commonappdata/Eye Saver/bin/fixgammaramp.exe` | .NET — gamma ramp for blue filter / brightness |
| `commonappdata/Eye Saver/temp/just_installed_flag.txt` | installer flag |
| `app/images/uninstall-warning.bmp` | asset |
| `tmp/NDP452-KB2901954-Web.exe` | .NET 4.5.2 web installer stub |

Also extracted PE shell via 7z to `/tmp/eyesaver_extract` (installer outer PE).

### Features recovered (strings / managed structure)
**Breaks**
- Enable breaks, interval (default pattern ~20 min), duration
- Break popup (`Eye_Saver.BreakPopup`) with progress bar
- Skip break, Delay 1 / 5 minutes, optional “disable skipping for few seconds”
- Reset timers after idle time

**Micro pauses**
- Enable/disable, interval, duration, sound notification, preview button

**Screen effects / blue light**
- Blue filter (color temperature) + `fixgammaramp.exe`
- Grayscale, Invert, Overlay effects (+ Overlay settings window)
- Auto set max brightness (laptops), day/night brightness values in settings strings
- Screen modes settings file

**Reminders**
- Stretch: *“Sitting too long is not healthy. Do some stretching.”*
- Water: *“Drink more water!”*
- Configurable “what reminders to check”

**System / UX**
- System tray, single-instance (*“Eye Saver is already running…”*)
- Autostart with Windows (→ Linux: login item)
- Prevent sleeping
- Automation rules window
- Licensing window (`Eye Saver - License`) — commercial/Leosoft
- Updater
- Settings persistence (user settings + screen modes file)

**UI names (WPF):** MainWindow, BreakPopup, Prompt, AutomationRulesWindow, LicensingWindow, OverlaySettingsWindow, DisabledAppMsg, etc.

### REA CLI commands used
```bash
rea inspect-artifact <exe> --json
rea inspect-pe-imports <exe> --json
rea inspect-managed-artifact 'Eye Saver.exe' --json
```
Both main exes: `mscoree.dll → _CorExeMain` (.NET).

---

## 6. Linux remake (built this session)

### Location
`/home/shuvo/Opencode/Eye_Saver/linux-app/`

### Stack
- **Electron ^33** + **electron-builder ^25**
- Node 24 local, Electron binary present (`node_modules/electron/dist/electron`)
- Syntax checked: `src/main.js`, `src/preload.js`, `src/renderer.js` → OK

### Files
| File | Role |
|------|------|
| `package.json` | scripts: `start`, `dist`, `dist:AppImage`, `dist:deb`, `dist:all` |
| `src/main.js` | tray, break/micro timers, settings IPC, overlay (blue/gray/invert), powerSaveBlocker, autostart, idle reset (xprintidle) |
| `src/preload.js` | `window.eyeSaver` contextBridge API |
| `src/index.html` + `styles.css` + `renderer.js` | settings UI (Breaks / Screen / Reminders / System tabs) |
| `src/break.html` | fullscreen break/micro-pause overlay (timer, delay 1/5, skip) |
| `.github/workflows/build.yml` | **ubuntu-latest** build → AppImage+deb artifact; tag `v*` → GitHub Release |

### Feature parity mapping
| Original | Linux implementation |
|----------|----------------------|
| Break timer + popup | `createBreakWindow('break')` + `break.html` |
| Micro pauses + sound | `createBreakWindow('micro')` |
| Blue filter | warm overlay window (`blueFilterIntensity`) |
| Grayscale / invert / overlay | CSS filter / tint overlay windows |
| Stretch / water reminders | `pickReminder()` |
| Idle reset | `xprintidle` every 30s |
| Prevent sleep | `powerSaveBlocker` |
| Autostart | `app.setLoginItemSettings` |
| Tray | `Tray` + menu (breaks/micro/blue toggles) |
| Single instance | `requestSingleInstanceLock` |
| Settings file | Electron `userData` → `settings.json` |

### Local run
```bash
export PATH="$HOME/tools/jdk25/bin:$HOME/.local/share/fnm:$HOME/.local/bin:$PATH"
eval "$(fnm env --shell bash)"
cd /home/shuvo/Opencode/Eye_Saver/linux-app
npm start
# or package:
npm run dist:all   # → dist/*.AppImage, dist/*.deb
```

### GitHub Actions
Workflow: `linux-app/.github/workflows/build.yml`
- Trigger: push to `main`/`master`, tags `v*`, or manual dispatch
- Steps: checkout → Node 22 → `npm ci`/`npm install` → `npm run dist:all` → upload artifacts
- On tag: softprops release with AppImage/deb

Repo root git initialized (`main` branch), `.gitignore` excludes:
- `Eye_Saver-setup-2.48.exe`
- `linux-app/node_modules/`, `linux-app/dist/`
- backups / extracts

**Not yet committed or pushed.** `gh` installed but **not logged in**.

---

## 7. Recommended next steps

1. **Restart OpenCode** (this session ends) — REA MCP + skill load on new session.
2. Confirm REA tools in new session:
   ```
   Check that REA is connected and show me which REA tools you can use.
   ```
   or `npx -y rea-agents@latest doctor --client opencode --json`.
3. **GitHub push:**
   ```bash
   gh auth login
   cd ~/Opencode/Eye_Saver
   git add -A
   git commit -m "Eye Saver Linux remake (Electron) + REA analysis notes"
   gh repo create eye-saver-linux --public --source=. --push
   # or: git remote add origin ... && git push -u origin main
   ```
4. Watch Actions build; download AppImage/deb from artifact or release.
5. Optional deeper REA: open `Eye Saver.exe` with Ghidra provider for native helpers (most logic is managed/.NET — `inspect-managed_artifact` / decompile managed types preferred).
6. Optional: `npm install --global rea-agents` for a permanent `rea` CLI + `rea update`.

---

## 8. Key paths cheat-sheet

```
REA MCP config:     ~/.config/opencode/opencode.jsonc
REA backup:         ~/.config/opencode/opencode.jsonc.rea.backup
REA skill:          ~/.agents/skills/reverse-engineer-anything/
REA npx cache:      ~/.npm/_npx/af752b90495482d7/
REA source clone:   ~/tools/rea/
Ghidra:             ~/tools/ghidra_12.2_PUBLIC/
JDK 25:             ~/tools/jdk25/
Node (fnm):         ~/.local/share/fnm/node-versions/v24.22.3/
Stable node:        ~/.local/bin/node
gh:                 ~/.local/bin/gh
Original installer: ~/Opencode/Eye_Saver/Eye_Saver-setup-2.48.exe
Linux app:          ~/Opencode/Eye_Saver/linux-app/
Extracted app:      /tmp/eyesaver_inno/app/Eye Saver.exe   (tmp — may vanish)
innoextract:        /tmp/innoextract-1.9-linux/bin/amd64/innoextract
```

---

## 9. Session decisions (user-approved)

- Node manager: **fnm** + Node 24 LTS
- Analysis engine: **Ghidra** (not Hopper)
- Linux stack: **Electron + GitHub Actions**
- REA setup: dry-run → user approval → apply for **OpenCode only**
- Build via **GitHub**, not local native packaging only

---

## 10. Security / hygiene notes

- Context7 API key exists in `opencode.jsonc` (pre-existing; not added by REA).
- Do not commit `Eye_Saver-setup-2.48.exe` or npx cache paths as secrets.
- REA analysis is local; model provider data policy still applies to tool results.
- Original Eye Saver is commercial (Leosoft) — Linux remake is a functional recreation for personal use; respect licensing if redistributing.
