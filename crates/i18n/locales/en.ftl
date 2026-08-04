# Base catalog. Ships inside the binary and is the fallback when the master
# has no bundle for a locale, or the launcher is offline.

## Window chrome
lang-ru = RU
lang-en = ENG

## Navigation
nav-game = GAME
nav-mods = MODS
nav-settings = SETTINGS
nav-news = NEWS
nav-profile = PROFILE

## Sidebar
sidebar-online = ONLINE
sidebar-offline = OFFLINE
sidebar-empty = No servers yet. Add one in the admin panel.
sidebar-servers = { $count ->
        [one] { $count } server
       *[other] { $count } servers
    }

## Game
game-no-servers = No servers
game-no-servers-hint = Add a server in the admin panel to get started.

## Profile
profile-title = PROFILE
profile-unavailable = Profile unavailable.
profile-sign-out = Sign out
profile-cape = Cape
profile-not-set = Not set
profile-edit = Edit
profile-upload-skin = Upload Skin
profile-drag-to-rotate = Drag to rotate
profile-skin-loading = Loading...
profile-no-skin = No skin set
profile-skin-invalid = No valid skin selected

## Sync and launch
sync-checking = Checking files...
sync-java = Downloading Java...
sync-minecraft = Downloading Minecraft...
sync-libraries = Downloading libraries...
sync-assets = Downloading assets...
sync-mods = Downloading mods...
sync-forge = Applying Forge patches...
sync-cleaning = Cleaning extra files...
sync-done = Done
sync-files-left = { $count ->
        [one] { $count } file left
       *[other] { $count } files left
    }

## Errors and notices
error-game-exited = Game exited with an error
error-sign-in-cancelled = Sign in cancelled
error-background-failed = Server background failed to load: { $reason }

## Notifications from the master
notif-server-error = Server error: { $reason }
notif-no-server-access = No access to this server
notif-no-published-build = This server has no published build
notif-update-failed = Update failed: { $reason }
notif-skin-upload-failed = Skin upload failed: { $reason }
notif-sign-in-to-upload = Sign in required to upload skin
notif-already-running = Game is already running

## Auth failures
auth-banned = Your account is banned
auth-session-expired = Session not found or expired
auth-sign-in-first = Sign in first

## Login
login-title = NORO LAUNCHER
login-subtitle = DISCORD OAUTH
login-sign-in = SIGN IN WITH DISCORD
login-waiting = WAITING...
login-checking = CHECKING SESSION...
login-save-session = Save session
login-auto-login = Auto login

## Game bar
game-build = BUILD
game-start = START GAME
game-stop = STOP
game-preparing = PREPARING
game-locked = LOCKED
game-vip-only = VIP ONLY
toast-success = DONE
toast-warning = HEADS UP
toast-error = ERROR
toast-info = NOTICE
game-node-offline = offline
game-online-unknown = online unknown
sync-failed = SYNC FAILED

## News
news-title = NEWS
news-empty = No news yet.

## Mods
mods-optional = OPTIONAL MODS
mods-empty = No optional mods configured for this server.

## Settings
settings-title = LAUNCHER SETTINGS
settings-client-title = CLIENT SETTINGS
settings-memory = JVM MEMORY
settings-memory-default = DEFAULT JVM MEMORY
settings-memory-hint = Allocated RAM for Minecraft.
settings-jvm-flags = JVM FLAGS
settings-jvm-hint = Extra JVM arguments passed at launch.
settings-folder = FOLDER
settings-folder-hint = The folder containing all instance files.
settings-folder-open = Open
settings-console = GAME CONSOLE
settings-console-hint = Log output from the game process.
settings-console-open = Open on launch
settings-console-show = Show console window on game launch
settings-reset = RESET
settings-update = LAUNCHER UPDATE
settings-install-update = Install Update
settings-source-default = LAUNCHER DEFAULT
settings-source-override = LOCAL OVERRIDE
settings-source-recommended = MASTER RECOMMENDED
settings-server = Server
console-title = GAME CONSOLE
settings-update-hint = A newer launcher version is available.
