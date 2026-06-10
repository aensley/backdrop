'use strict'

// On Linux, install a .desktop file so GNOME Wayland's compositor can match
// the running window (app_id = com.andrewensley.backdrop) to an icon.
// On other platforms this is a no-op.
if (process.platform !== 'linux') process.exit(0)

const fs = require('fs')
const path = require('path')
const { execSync } = require('child_process')

const projectRoot = path.resolve(__dirname, '..')
const iconPath = path.join(projectRoot, 'src-tauri', 'icons', 'icon.svg')
const appId = 'com.andrewensley.backdrop'
const appsDir = path.join(process.env.HOME, '.local', 'share', 'applications')
const desktopFile = path.join(appsDir, `${appId}.desktop`)

const content =
  [
    '[Desktop Entry]',
    'Type=Application',
    'Name=backdrop',
    'Exec=backdrop',
    `Icon=${iconPath}`,
    'StartupWMClass=backdrop',
    'NoDisplay=true'
  ].join('\n') + '\n'

fs.mkdirSync(appsDir, { recursive: true })
fs.writeFileSync(desktopFile, content)

try {
  execSync(`update-desktop-database "${appsDir}"`, { stdio: 'ignore' })
} catch (_) {
  // update-desktop-database may not be present; not required
}

console.log(`[dev-setup] installed ${desktopFile}`)
