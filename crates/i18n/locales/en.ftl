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
sidebar-signed-out = Not signed in
sidebar-no-discord = No Discord account

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
profile-skin-untitled = Untitled skin
profile-skin-picker-failed = Could not open the file picker
profile-skin-unreadable = Could not read that file
profile-skin-not-png = That file is not a PNG image
profile-skin-too-large = Skin file must be under 256 KB
profile-presets-title = Skin presets
profile-preset-new = New skin
profile-preset-upload-png = Upload .PNG
profile-preset-upload = Upload
profile-preset-current = Current
profile-preset-wear = Wear
profile-cape-title = Choose a cape
profile-cape-remove = Remove cape
profile-cape-none = No cape
profile-cape-take-off = Take off

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
retry = RETRY

## Notifications from the master
notif-server-error = Server error: { $reason }
notif-no-server-access = No access to this server
notif-no-published-build = This server has no published build
notif-build-files-restored = Build files were restored
notif-launch-blocked = Launch blocked: a banned file was found in the game folder
notif-support-sent = Logs sent — thank you
impersonate-title = SIGN IN AS A PLAYER
logreq-title = AN ADMIN IS ASKING FOR LOGS
notif-remote-action-done = Done: { $detail }
remote-action-title = AN ADMIN IS ASKING TO RUN SOMETHING
remote-action-clear_asset_cache = Clear the asset cache
remote-action-reinstall_build = Reinstall the build
remote-action-restart_launcher = Restart the launcher
remote-action-verify_integrity = Verify integrity
remote-action-accept = Allow
remote-action-decline = Decline
logreq-title-forced = LOGS COLLECTED ON REQUEST
logreq-not-collected = Worlds, screenshots and the server list are never collected. Your username and tokens are stripped out.
logreq-preview = See what will be sent
logreq-send = Send
logreq-decline = Decline
logreq-close = Close
impersonate-accept = Sign in
impersonate-decline = Decline
impersonate-banner = Signed in as
impersonate-exit = Exit
notif-impersonate-failed = Could not sign in: { $reason }
notif-support-failed = Could not send logs: { $reason }
notif-support-nothing-to-send = No logs yet — start the game first
notif-sign-in-first = Sign in first
notif-update-failed = Update failed: { $reason }
notif-skin-upload-failed = Skin upload failed: { $reason }
notif-sign-in-to-upload = Sign in required to upload skin
notif-sign-in-to-suggest = Sign in required to request a mod
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
login-tagline = A Minecraft server launcher that installs mods and skins for you
login-sign-in-discord = Sign in with Discord
login-sign-in-passkey = Sign in on the website (Passkey)

## Game bar
game-build = BUILD
game-start = START GAME
game-install = INSTALL
game-update = UPDATE
game-stop = STOP
game-preparing = PREPARING
game-locked = LOCKED
game-vip-only = FRIEND ONLY
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
news-back = Back
news-read = Read

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
settings-fullscreen = FULLSCREEN MODE
settings-fullscreen-hint = Launch Minecraft in fullscreen mode.
settings-fullscreen-show = Launch in fullscreen mode
settings-reset = RESET
settings-update = LAUNCHER UPDATE
settings-install-update = Install Update
settings-source-default = LAUNCHER DEFAULT
settings-source-override = LOCAL OVERRIDE
settings-source-recommended = MASTER RECOMMENDED
settings-server = Server
console-title = GAME CONSOLE
settings-update-hint = A newer launcher version is available.
settings-crash-reports = CRASH REPORTS
settings-crash-reports-hint = Send an anonymous report when the launcher crashes. No account or machine name is included. Takes effect after a restart.
settings-support-bundle = REPORT A PROBLEM
settings-support-bundle-hint = Send the last session's logs to the admins. Worlds, screenshots and the server list are never collected, and your username and tokens are stripped out.
settings-support-send = Send logs
