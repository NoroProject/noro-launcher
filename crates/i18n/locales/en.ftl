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
game-build-preview = PREVIEW
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

# ---------------------------------------------------------------------------
# Website. The `web-` prefix keeps site strings apart from launcher ones: the
# admin editor filters by it, and the launcher never asks for these keys.
# ---------------------------------------------------------------------------

## Navigation and footer
web-nav-home = Home
web-nav-servers = Servers
web-nav-rules = Rules
web-nav-cabinet = Cabinet
web-nav-sign-in = Sign in
web-nav-menu-open = Open menu
web-nav-menu-close = Close menu
web-footer-tagline = Modded Minecraft project
web-footer-legal = © { $year } Noro. Not affiliated with Mojang or Microsoft.

## Home
web-home-online-now = { $count ->
        [one] { $count } player online now
       *[other] { $count } players online now
    }
web-home-lead = A convenient Minecraft modded server project: one-click Discord login, automatic mod downloads, custom skins, and capes all in one place.
web-home-cta-cabinet = OPEN CABINET
web-home-cta-discord = START WITH DISCORD
web-home-stat-online = Online
web-home-stat-servers = Servers
web-home-worlds-eyebrow = Worlds
web-home-worlds-title = Servers of the project
web-home-worlds-all = All servers
web-home-feature-builds-title = Fast Mod Setup
web-home-feature-builds-text = The launcher automatically downloads and updates all required mods in seconds — just click "Play".
web-home-feature-identity-title = Unified Account
web-home-feature-identity-text = One-click Discord login, skins, capes, and personal cabinet all tied to your account.
web-home-feature-rules-title = Fair Play
web-home-feature-rules-text = Clear rules and transparent moderation — the rulebook is public and easy to navigate.
web-home-flow-eyebrow = Launch flow
web-home-flow-title = From Discord to Minecraft in four steps
web-home-step-signin-title = Sign in
web-home-step-signin-text = Log in via Discord on the website — your profile is instantly ready for game.
web-home-step-server-title = Pick a server
web-home-step-server-text = Choose your favorite modpack and server from the list.
web-home-step-sync-title = Auto-download
web-home-step-sync-text = The launcher automatically downloads and updates all required files.
web-home-step-play-title = Launch & play
web-home-step-play-text = Click play — Minecraft launches fully pre-configured and ready.
web-home-rules-title = Read the Server Rules
web-home-rules-text = Our rules are clear and simple to ensure a welcoming, friendly game environment for everyone.
web-home-rules-cta = Open rules

## Servers
web-servers-meta-title = Servers — Noro
web-servers-meta-description = Modded Minecraft servers of the project: versions, addresses and live player counts.
web-servers-title = Servers
web-servers-lead = Every world of the project with its version, address and how many players are on it right now.
web-servers-online = Players online
web-servers-refresh = Refresh
web-servers-loading = Loading servers…
web-servers-empty-title = No servers published yet
web-servers-empty-text = The project has not opened a world to the public yet. Check back later.
web-servers-empty-short = The project has not opened a world to the public yet.

## Server card
web-server-online = Online
web-server-offline = Offline
web-server-players = Players
web-server-rules = Rules
web-server-address-copied = Address copied

## Launcher download
web-download-title = GET THE LAUNCHER
web-download-lead = Sign in with Discord, pick a server, and the launcher syncs the rest.
web-download-loading = Loading builds…
web-download-none = No launcher build published yet.
web-download-for = Download for { $platform }
web-download-other = Other platforms
web-download-signed = Every build is signed — the launcher checks the signature before it runs.

## Rules
web-rules-meta-title = Rules — Noro
web-rules-meta-description = Project rules: what is allowed on the servers and what follows if it is not.
web-rules-title = Rules
web-rules-lead = Every punishment cites a rule by its code. Search by code, title or wording — the same codes are used in game, in bans and in tickets.
web-rules-total = Rules in this book
web-rules-search = Search by code (1.1), title or wording
web-rules-search-aria = Search rules
web-rules-scope = Scope
web-rules-scope-general = General
web-rules-found = { $count ->
        [one] { $count } rule matches “{ $query }”
       *[other] { $count } rules match “{ $query }”
    }
web-rules-loading = Loading rules…
web-rules-failed-title = Rules are unavailable
web-rules-failed-text = The master server did not answer. Try again in a minute.
web-rules-nomatch-title = Nothing matches
web-rules-nomatch-text = No rule mentions “{ $query }”. Try a code like 1.1 or a single word.
web-rules-empty-title = No rules published yet
web-rules-empty-text = The team has not published the rulebook. Check back later.
web-rules-other = Other rules
web-rules-contents = Contents
web-rules-count = { $count ->
        [one] { $count } rule
       *[other] { $count } rules
    }
web-rules-copy-link = Copy link to this rule
web-rules-link-copied = Link copied
web-rules-link-copied-body = Rule { $code }

## Punishments a rule allows
web-sanction-warn = Warning
web-sanction-mute = Mute
web-sanction-ban = Ban
web-sanction-server-ban = Server ban
web-sanction-possible = Possible punishment
web-sanction-exact = { $kind } { $min }
web-sanction-range = { $kind } { $min }–{ $max }
web-sanction-from = { $kind } from { $min } to permanent
web-sanction-upto = { $kind } up to { $max }
web-sanction-any = { $kind } any duration or permanent

## Cabinet: punishments
cabinet-punishments-title = Punishments & Warnings
cabinet-punishments-lead = Full history of warnings, bans, mutes, and server restrictions
cabinet-punishments-refresh = Refresh
cabinet-punishments-none-title = No punishments
cabinet-punishments-none-text = You have no active or past warnings or bans on record.

## Punishment kinds & status badges
punishment-kind-ban = BAN
punishment-kind-warn = WARN
punishment-kind-mute = MUTE
punishment-kind-server-ban = SERVER BAN
punishment-status-active = ACTIVE
punishment-status-expired = EXPIRED
punishment-status-revoked = REVOKED
punishment-actor = Issued by: { $actor }
punishment-until = until { $date }
punishment-forever = forever

## Admin punishments panel
admin-punish-title = Punishments
admin-punish-empty = Nothing on record.
admin-punish-revoked = revoked
admin-punish-expired = expired
admin-punish-allowed-for-rule = Allowed for this rule
admin-punish-none-allowed = This rule sets nothing you are allowed to issue.
admin-punish-kind = Kind
admin-punish-target-server = Target server
admin-punish-target-server-optional = (optional)
admin-punish-target-all-servers = All servers
admin-punish-target-select-server = Select server…
admin-punish-reason = Reason
admin-punish-reason-placeholder = What explains this punishment in six months
admin-punish-bypass-hint = bypass: rule limits do not apply to you
admin-punish-rule-label = Rule
admin-punish-rule-clear = Clear rule
admin-punish-rule-search = Search the rulebook by code or wording
admin-punish-rule-empty = The rulebook is empty.
admin-punish-rule-nomatch = Nothing matches this search.
admin-punish-duration-label = Duration
admin-punish-duration-hint-empty = empty = forever
admin-punish-duration-hint-until = { $duration } — until { $until }

## Navigation & Sidebar
nav-admin-control = Admin Control
nav-player-cabinet = Player Cabinet
nav-switch-to-cabinet = Player Cabinet
nav-switch-to-admin = Admin Panel
nav-cabinet-home = Cabinet
nav-cabinet-skin = Skin
nav-cabinet-punishments = Punishments
nav-cabinet-rules = Rules
nav-cabinet-apps = Apps
nav-cabinet-settings = Settings
nav-group-management = Management
nav-group-content = Content
nav-group-system = System

## Admin user notes
admin-notes-title = Notes
admin-notes-subtitle = Admins only — the player never sees these.
admin-notes-placeholder = What happened…
admin-notes-add = Add
admin-notes-empty = No notes yet.

## Cabinet index page
cabinet-title = Cabinet
cabinet-subtitle = Profile and access
cabinet-player = Player
cabinet-mc-name = Minecraft name
cabinet-mc-name-hint = Shown to other players in game. Up to 16 characters.
cabinet-save = Save
cabinet-profile-updated = Profile updated
cabinet-passkeys-title = Passkeys (WebAuthn)
cabinet-passkeys-lead = Passwordless login with Touch ID, Face ID, or security keys
cabinet-passkeys-add = Add Passkey
cabinet-passkeys-created = Created { $date }
cabinet-passkeys-used = last used { $date }
cabinet-passkeys-unused = never used
cabinet-passkeys-none-title = No registered Passkeys
cabinet-passkeys-none-text = Add a Touch ID or Face ID key for fast sign-in without Discord
cabinet-launcher-title = Launcher
cabinet-roles-title = Roles — { $count }
cabinet-perms-count = { $count } perms
cabinet-roles-none-title = No roles yet
cabinet-roles-none-text = Server access is granted through roles.
cabinet-direct-perms-title = Direct permissions — { $count }
cabinet-direct-none-title = Nothing granted directly
cabinet-direct-none-text = That is normal — access usually comes from roles.

## Authorized applications
cabinet-apps-title = Authorized Applications
cabinet-apps-subtitle = Manage applications with account access
cabinet-apps-lead = Third-party services and launchers with access to your profile
cabinet-apps-refresh = Refresh
cabinet-apps-default-desc = Access to your Noro Network profile
cabinet-apps-revoke = Revoke Access
cabinet-apps-none-title = No third-party applications connected
cabinet-apps-none-text = Applications and launchers you have granted access to will appear here

## OAuth2 Authorization
oauth-loading-app = Loading application details…
oauth-official-app = Official Application
oauth-app-access-request = Application is requesting access to your Noro Network account.
oauth-discord-connected = Discord: Connected
oauth-requested-permissions = Requested Permissions
oauth-scope-profile = User profile & in-game username ({ $scope })
oauth-deny = Deny
oauth-allow = Allow Access

## Skin and Capes Manager
skin-title = Skins & Capes
skin-subtitle = Modrinth & Pandora style skin manager
skin-3d-character = 3D Character
skin-custom-badge = Custom Skin
skin-default-badge = Default
skin-reset-default = Reset to Default
skin-your-skins = Your skins and presets
skin-drop-hint = Click the plus card or drag a file to create a preset
skin-new-skin = New skin
skin-upload-png = Upload a .PNG file
skin-equipped = Equipped
skin-equip = Equip
skin-official-skins = Official Minecraft skins
skin-mojang-desc = Standard Mojang characters
skin-available-capes = Your available capes
skin-pick-cape-desc = Pick a cape for your character
skin-capes-count = { $count } capes
skin-no-cape = No cape
skin-no-capes-title = No capes available
skin-no-capes-desc = You have no capes yet. Ask an administrator to grant you cape access.

## Login page
login-secure-login = Secure login
login-discord = SIGN IN WITH DISCORD
login-passkey = SIGN IN WITH PASSKEY
login-recovery-toggle = Use a recovery code
login-username-placeholder = Username
login-submit = SIGN IN
login-recovery-hint = Each code works once. After signing in, bind a passkey — that is what the next login should rest on.

## Admin: blocklist
admin-blocklist-title = Blocklist
admin-blocklist-subtitle = Files that must not be in a game folder
admin-blocklist-note = SHA1 is defeated by changing one byte, a name mask by renaming. The list ships inside the signed manifest, so it cannot be swapped out on the client.
admin-blocklist-mask = Name mask
admin-blocklist-sha1 = SHA1
admin-blocklist-reason = Reason
admin-blocklist-action = Action
admin-blocklist-act-delete = Delete
admin-blocklist-act-flag = Flag only
admin-blocklist-act-block = Block launch
admin-blocklist-empty-title = Nothing blocked
admin-blocklist-empty-text = Add a mask or a hash — the rules ship inside the signed manifest.

## Admin: rule & category modals
admin-rule-edit = Edit rule { $code }
admin-rule-new = New rule
admin-rule-code = Code
admin-rule-section = Section
admin-rule-no-section = No section
admin-rule-server-only = { $name } only
admin-rule-wording = Wording
admin-rule-create = Create rule
admin-cat-edit = Edit section { $name }
admin-cat-new = New section
admin-cat-number = Number
admin-cat-parent = Parent section
admin-cat-top = Top level
admin-cat-intro = Intro (optional)
admin-cat-create = Create section
admin-sanc-title = Possible punishments
admin-sanc-add-option = Option
admin-sanc-no-limits = No limits set: only moderators with noro.mod.punish.bypass will be able to punish under this rule.
admin-sanc-kind = Kind
admin-sanc-from = From
admin-sanc-to = Up to
admin-sanc-note = Note (optional)

## Admin: launcher versions table
admin-launchver-plat = Platform
admin-launchver-kind = Kind
admin-launchver-curr = Current
admin-launchver-builds = { $count } builds
admin-launchver-deploy-core = Deploy core
admin-launchver-deploy-boot = Deploy bootstrap
admin-launchver-is-curr = current
admin-launchver-stored = stored
admin-launchver-deploy = Deploy

## Admin: translations
admin-i18n-title = Translations
admin-i18n-subtitle = Launcher text
admin-i18n-changed-count = { $count } of { $total } changed
admin-i18n-hint = Leave a field empty to use the built-in text shown next to it. Only what you fill in is sent to launchers, so untouched keys keep working after updates.
admin-i18n-search-placeholder = Search key or text
admin-i18n-only-changed = Only changed
admin-i18n-col-key = Key
admin-i18n-col-builtin = Built-in
admin-i18n-col-override = Override
admin-i18n-no-match = Nothing matches the filter.
admin-i18n-unsaved = Unsaved changes
admin-i18n-reset-all = Reset all to built-in

## Admin: settings
admin-settings-title = Settings
admin-settings-subtitle = Instance configuration
admin-settings-export-env = Export .env
admin-settings-secrets-title = Secrets
admin-settings-secrets-lead = Read-only by design. Secrets live in the environment only.
admin-settings-secret-set = set in the environment
admin-settings-secret-unset = not set
admin-set-label-instance_name = Instance name
admin-set-label-public_url = API URL
admin-set-hint-public_url = Ends up in every manifest a player downloads.
admin-set-label-web_url = Site URL
admin-set-hint-web_url = Passkeys are bound to this domain permanently.
admin-set-label-allowed_origins = Allowed CORS origins
admin-set-label-hero_image_url = Hero illustration URL
admin-set-hint-hero_image_url = Main page character render image URL. Updates instantly without restart.
admin-settings-hero-title = Hero Render Illustration
admin-settings-hero-desc = Main page character illustration. Updates instantly when a file is uploaded without server restart.
admin-settings-hero-upload = Upload image
admin-set-hint-allowed_origins = Comma-separated. Empty means any origin is accepted.
admin-set-label-discord_client_id = Discord Client ID
admin-set-label-files_cdn_url = CDN URL for files
admin-set-label-github_repo = GitHub repository
admin-set-label-github_ref = GitHub branch
admin-set-label-launcher_repo = Local launcher checkout
admin-set-from-env = from env
admin-set-from-env-title = Set by { $env }. The environment wins over the database.
admin-diag-panel-desc = Things that otherwise only ever show up as one line in the startup log.

## Admin: support & logs
admin-support-title = Support Logs
admin-support-subtitle = Received client support bundles and log collection requests
admin-support-bundles-title = Delivered Bundles ({ $count })
admin-support-archives-count = { $count } archives
admin-support-user = User { $id }
admin-support-voluntary = Voluntary
admin-support-forced = Forced
admin-support-date = Received { $at } · Expires { $expires }
admin-support-download-zip = Download ZIP
admin-support-nobundles-title = No log bundles
admin-support-nobundles-text = No client support bundles received yet.
admin-support-requests-title = Log Requests ({ $count })
admin-support-requests-count = { $count } requests
admin-support-norequests-title = No log requests
admin-support-norequests-text = No log requests sent yet.

## Admin: roles & permissions
admin-roles-title = Roles
admin-roles-subtitle = ACL through glob permissions
admin-roles-new-role = New role
admin-roles-col-role = Role
admin-roles-col-perms = Permissions
admin-roles-col-default = Default
admin-roles-yes = yes
admin-roles-no = no
admin-roles-modal-title = NEW ROLE
admin-roles-modal-subtitle = Group permissions for launcher users
admin-roles-name = Name
admin-roles-display-name = Display name
admin-roles-color = Color
admin-roles-order = Order
admin-roles-is-default = Default role
admin-role-back = Back
admin-role-icon-hint = A single character shown next to the name. Unicode works everywhere — in the cabinet, in the launcher and in game chat.
admin-role-inherits-label = Inherits from
admin-role-inherits-none = Nothing — own permissions only
admin-role-inherits-hint = Everything the parent grants applies here too, all the way up the chain.
admin-role-lp-label = LuckPerms group
admin-role-lp-hint = Links this role to a group in game. Leave empty if the role should not reach the server.
admin-role-perms-title = Role permissions
admin-role-perms-subtitle = Everyone in this role gets them. Pick the builds a permission applies to, or all of them.

## Admin: news
admin-news-title = News
admin-news-subtitle = Markdown posts for launcher
admin-news-new-post = New post
admin-news-pinned = pinned
admin-news-empty-title = No news yet
admin-news-empty-text = Create the first post from the toolbar.
admin-news-modal-title = NEW POST
admin-news-modal-subtitle = Publish launcher news in Markdown
admin-news-post-title = Title
admin-news-post-body = Body
admin-news-post-pinned = Pinned
admin-news-preview = Preview

## Admin: launcher releases
admin-launch-title = Launcher
admin-launch-subtitle = Versions, GitHub tag builds, and deploy
admin-launch-build-tag = Build tag
admin-launch-build-log = Build log
admin-launch-empty-title = No versions yet
admin-launch-empty-text = Build a launcher tag from the toolbar.
admin-launch-modal-title = GITHUB BUILD
admin-launch-modal-subtitle = Build a launcher release tag
admin-launch-check-release = Check latest release

## Admin: API & CLI tokens
admin-tokens-title = Tokens
admin-tokens-subtitle = Tokens for CLI and CI
admin-tokens-new-token = New token
admin-tokens-secret-once = Secret is shown once
admin-tokens-last-used = Last used
admin-tokens-empty-title = No tokens yet
admin-tokens-empty-text = Create a CLI token from the toolbar.
admin-tokens-modal-title = NEW TOKEN
admin-tokens-modal-subtitle = Secret will be shown once
admin-tokens-perms-label = Permissions, one per line

## User moderation
admin-users-version = Version
admin-users-login-as = Login as
admin-users-req-logs = Request logs
admin-users-end-sessions = End all sessions
admin-users-impersonate-title = Sign in as this player
admin-users-impersonate-warn = The player is not notified. Everything you change while signed in as them is recorded in the audit log with your name.
admin-users-impersonate-code = Recovery code
admin-users-impersonate-stepup = Confirm it is you. The confirmation then holds for { $mins } minutes.
admin-users-impersonate-passkey = Confirm with passkey
admin-users-impersonate-use-code = No passkey here? Use a recovery code
admin-users-impersonate-use-passkey = Use a passkey instead
admin-users-impersonate-confirm = Confirm identity
admin-users-impersonate-request = Request access to { $name }
admin-users-reqlogs-title = Request logs
admin-users-reqlogs-target = Target Server / Build
admin-users-reqlogs-auto = Auto (Current / Launcher Root)
admin-users-reqlogs-why = Why
admin-users-reqlogs-why-hint = The player sees this text, and it stays in the audit log.
admin-users-reqlogs-force = Collect without asking.
admin-users-reqlogs-force-hint = Recorded separately in the audit log.
admin-users-reqlogs-btn-now = Collect now — { $name }
admin-users-reqlogs-btn-ask = Ask — { $name }

## Common web keys & punishments
web-rules-cancel = Cancel
web-rules-save = Save
web-rules-status-online = Online
web-rules-status-offline = Offline
punish-warn = Warning
punish-mute = Mute
punish-ban = Ban
punish-server-ban = Server ban

## Admin sidebar navigation
nav-admin-dashboard = Dashboard
nav-admin-clients = Clients
nav-admin-servers = Servers
nav-admin-mods = Mods
nav-admin-users = Users
nav-admin-capes = Capes
nav-admin-roles = Roles
nav-admin-integrity = Integrity
nav-admin-blocklist = Blocklist
nav-admin-news = News
nav-admin-rules = Rules
nav-admin-translations = Translations
nav-admin-wrapper = Wrapper
nav-admin-launcher = Launcher
nav-admin-tokens = Tokens
nav-admin-audit = Audit Log
nav-admin-support = Support Logs
nav-admin-settings = Settings

## Admin builds and game servers
admin-gs-title = Game servers
admin-gs-subtitle = Instances running this pack. Backends report the player count and run the agent; a proxy is where players connect.
admin-gs-add = Add server
admin-gs-host = Host
admin-gs-port = Port
admin-gs-backend = Backend
admin-gs-proxy = Proxy
admin-gs-create = Create
admin-gs-empty-title = No game servers registered
admin-gs-empty-text = Add one to get an agent secret and see the online count.
admin-gs-never-seen = never seen
admin-gs-just-now = just now
admin-gs-ago = { $time } ago
admin-gs-no-address = no address
admin-gs-control-tooltip = Control: console, files, mods
admin-gs-rotate-tooltip = Issue a new secret — the old one stops working
admin-set-profile-title = Profile
admin-set-profile-text = How this pack is named and where it shows up.
admin-set-client-title = Client defaults
admin-set-client-text = Version and loader new builds inherit from this pack.
admin-set-active = Active
admin-set-active-hint = Show this pack in launcher lists.
admin-set-limited = Limited access
admin-set-limited-hint = Require role access before players can join.
admin-set-client-defaults-hint = Builds inherit these defaults when a new build is created from this pack. Server addresses live on the game servers below.
admin-builds-title = Builds
admin-builds-subtitle = Manage game versions, files, mods, and publish state.
admin-builds-col-build = Build
admin-builds-col-status = Status
admin-builds-published = published
admin-builds-draft = draft
admin-builds-empty-title = No builds yet
admin-builds-empty-text = Create the first build from the toolbar.

## Server media assets
admin-media-title = Server assets
admin-media-subtitle = Icon and banner used by the launcher profile.
admin-media-banner-upload = Upload banner
admin-media-banner-hint = Drop a wide image or click to browse.
admin-media-icon = Icon
admin-media-icon-upload = Upload icon

## Build management panels
admin-optmods-title = Optional Mods
admin-optmods-configured = { $count } mods configured
admin-optmods-allow-suggestions = Allow Mod Suggestions
admin-optmods-save = Save Optional Mods
admin-optmods-empty-title = No optional mods
admin-optmods-empty-text = Players can enable optional components.
admin-optmods-untitled = Untitled Optional Mod
admin-optmods-display-name = Display Name
admin-optmods-category = Category
admin-optmods-author = Author
admin-optmods-icon-url = Icon URL
admin-optmods-files-csv = Files (CSV)
admin-optmods-behavior = Behavior
admin-optmods-enabled-default = Enabled by default
admin-optmods-visible = Visible in UI
admin-optmods-restricted = Restricted access
admin-paths-title = Path Rules
admin-paths-subtitle = Sync exclusions & overrides
admin-paths-rules-count = { $count } rules
admin-paths-ignored = Ignored
admin-paths-ignored-hint = Never touched by sync
admin-paths-user-overrides = User Overrides
admin-paths-user-overrides-hint = Seeded once, player edits kept
admin-paths-edit-fm = Edit in file manager
admin-recom-title = Recommended Client Settings
admin-recom-subtitle = Defaults sent to launcher for this build
admin-recom-min-mem = Min Memory
admin-recom-max-mem = Max Memory
admin-recom-show-console = Show console window on launch
admin-recom-jvm-flags = JVM Flags
admin-recom-save = Save Recommendations

## Catalog filters and sync rules
admin-facets-source = Source
admin-facets-both = Both
admin-facets-runs-on = Runs on
admin-facets-any-loader = Any loader
admin-facets-any-version = Any version
admin-facets-any-side = any
admin-facets-clear = Clear { $count } filter(s)
admin-syncmodal-title = Edit Sync Rules
admin-syncmodal-subtitle = Manually edit paths to ignore or keep player modifications
admin-syncmodal-ignored-paths = Ignored Paths (Unmanaged)
admin-syncmodal-user-overrides = User Overrides (User Managed)
admin-syncmodal-quick-add = Quick Add:
admin-syncmodal-ignored-hint = Files and folders in this list will never be downloaded or deleted by the launcher sync. Folders must end with /
admin-syncmodal-user-hint = Files in this list are downloaded once on first install, then never overwritten by launcher sync.
admin-syncmodal-save = Save Rules

## Build files, release publishing, wrapper setup
admin-filesummary-title = Build Files
admin-filesummary-count = { $count } files
admin-filesummary-hidden = (core assets hidden in preview)
admin-filesummary-open = Open File Manager
admin-filesummary-empty = No files yet. Use the panels on the right to add.
admin-publish-title = Release Build
admin-publish-subtitle = Bootstrap & Signature
admin-publish-draft = Draft Mode
admin-publish-live = Live Release
admin-publish-btn = Publish Build
admin-publish-rebuild = Rebuild Version
admin-publish-scratch = Rebuild from Scratch
admin-publish-revert = Revert to Draft
admin-publish-delete = Delete Build
admin-wrapper-setup-title = Set up a game server
admin-wrapper-setup-step1 = 1. Download the wrapper
admin-wrapper-setup-step1-hint = Put it next to your server jar.
admin-wrapper-setup-step2 = 2. Create noro-wrapper.properties
admin-wrapper-setup-step3 = 3. Start the server through it
admin-wrapper-setup-step4 = 4. Keep online-mode on

## User settings and support panel
cabinet-settings-account = Account
cabinet-settings-session = Session
cabinet-settings-signout-hint = Signing out only affects this browser. The launcher keeps its own session.
admin-support-panel-title = Support Logs & Remote Control
admin-support-panel-subtitle = Client logs, crash dumps, and remote actions
admin-support-close-game = Close Game
admin-support-restart-launcher = Restart Launcher
admin-support-delivered-bundles = Delivered Bundles ({ $count })
admin-support-nobundles-player = No support bundles received from this player yet.
admin-support-req-history = Request History ({ $count })
admin-support-norequests-player = No log requests have been sent to this player.

## Users list and user detail profile
admin-users-title = Users
admin-users-subtitle = Profiles, bans, roles, and direct permissions.
admin-users-player = Player
admin-users-discord = Discord
admin-users-roles = Roles
admin-users-status = Status
admin-users-banned = banned
admin-users-active = active
admin-users-empty-title = No users yet
admin-users-tab-profile = Profile & Permissions
admin-users-tab-skins = Skin & Capes Access
admin-users-tab-support = Support & Logs
admin-users-tab-mod = Moderation
admin-users-not-found = User not found
admin-users-direct-perms = Direct Permissions
admin-users-direct-perms-hint = Granted to this player on top of their roles. Pick the builds a permission applies to, or all of them.
admin-users-skin-preview = 3D Skin Preview
admin-users-custom-skin = Custom Skin
admin-users-uploaded = Uploaded
admin-users-default-skin = Default
admin-users-upload-skin = Upload Skin
admin-users-reset-skin = Reset Skin
admin-users-presets-title = Skin Presets
admin-users-presets-subtitle = Saved player skins gallery
admin-users-active-cape = Active Cape
admin-users-active-cape-subtitle = Select which granted cape is equipped on player's model
admin-users-no-cape = No cape (Disabled)
admin-users-granted-capes = Granted Capes Access
admin-users-granted-capes-subtitle = Toggle capes from server catalog allowed for this player to choose in Cabinet & Launcher

## Admin servers list
admin-servers-title = SERVERS
admin-servers-subtitle = Server profiles, order, and launch metadata
admin-servers-new = New server
admin-servers-col-stack = Stack
admin-servers-col-status = Status
admin-servers-empty-title = No servers yet
admin-servers-empty-text = Create your first server profile to get started.
admin-servers-create-title = NEW SERVER
admin-servers-create-subtitle = Create a server and its first build profile
admin-servers-name = Server Name
admin-servers-name-hint = Addresses are set per game server once the pack exists.
admin-servers-initial-build = Initial Build Configuration
admin-servers-build-version = Build Version
admin-servers-create-later = You can create builds later in the server settings.
admin-servers-create-btn = Create Server

## Permissions, roles, diagnostics and capes
admin-roles-select = Select role
admin-perm-context = Context
admin-perm-all-builds = All builds
admin-perm-permission = Permission
admin-perm-add = Add
admin-perm-suggestions-unavailable = Suggestions unavailable: { $error }. Permissions can still be typed by hand.
admin-perm-already-granted = Already granted everywhere you picked.
admin-perm-pick-build = Pick at least one build, or grant it on all of them.
admin-perm-no-permissions = No permissions granted yet.
admin-diag-title = Diagnostics
admin-diag-collect = Collect
admin-diag-subtitle = Versions, hardware and link speed — nothing personal.
admin-diag-empty = Nothing collected yet.
admin-diag-verify = Verify files
admin-diag-clear-assets = Clear assets
admin-diag-restart-launcher = Restart launcher
admin-capes-equip = Equip
admin-capes-access-granted = Access Granted
admin-capes-access-not-granted = Access Not Granted
admin-capes-selected = Selected
admin-capes-granted = Granted
admin-capes-locked = Locked
admin-capes-count-granted = { $count } / { $total } granted
admin-capes-presets-count = { $count } presets
admin-capes-empty-presets = Player hasn't saved any presets yet.

## Admin: Main Dashboard
admin-dash-title = ADMIN
admin-dash-subtitle = Master server operations dashboard
admin-dash-card-users = Users
admin-dash-card-servers = Servers
admin-dash-card-builds = Builds
admin-dash-card-online = Launchers online
admin-dash-data-state = Data state
admin-dash-filestore = FileStore Storage
admin-dash-backup-btn = Download database backup
admin-dash-backup-hint = A pg_restore archive of the master database — accounts, permissions, skins and capes.
admin-dash-quick-actions = Quick actions
admin-dash-create-client = Create client
admin-dash-publish-news = Publish news
admin-dash-deploy-launcher = Deploy launcher

## Admin: Build section tabs
admin-tab-profile = Profile
admin-tab-profile-hint = Name, order, assets & visibility
admin-tab-mods = Mods
admin-tab-mods-hint = Installed mods & Modrinth catalog
admin-tab-build = Build & Files
admin-tab-build-hint = Modpack import, files & publish
admin-tab-instances = Game Servers
admin-tab-instances-hint = Backend servers & wrappers
admin-tab-client = Client Defaults
admin-tab-client-hint = RAM, JVM flags & optional mods

## Admin: Build selector
admin-build-active = Active Build
admin-build-total = { $count } total
admin-build-subtitle = Select assembly build version to configure files, import packs, and publish.
admin-build-live = live
admin-build-draft = draft
admin-build-new = New Build

## Admin: Pack import
admin-import-title = Pack Import
admin-import-subtitle = Modrinth & CurseForge
admin-import-format = Pack Format
admin-import-drop = Click or Drop Pack File
admin-import-process = Process Pack

## Admin: Manual upload
admin-manual-title = Manual Upload
admin-manual-subtitle = Direct Artifact injection
admin-manual-drop = Drop File Here
admin-manual-path = Destination Path
admin-manual-add = Add to Build

## Admin: File Manager
admin-fm-title = File Manager
admin-fm-subtitle = Build file browser & sync rules
admin-fm-open = Open
admin-fm-edit = Edit
admin-fm-download = Download
admin-fm-rename = Rename
admin-fm-copy-path = Copy Path
admin-fm-sync-mode = Sync mode
admin-fm-new-folder = New Folder
admin-fm-upload = Upload Files
admin-fm-delete = Delete
admin-fm-col-name = Name
admin-fm-col-sync = Sync
admin-fm-col-size = Size
admin-fm-col-kind = Kind
admin-fm-items = items
admin-fm-folder = Folder
admin-fm-empty = Empty folder
admin-fm-sync-synced = Synced — server version always wins
admin-fm-sync-ignored = Ignored — never downloaded or removed
admin-fm-sync-user = User — installed once, then left alone

## Admin: Create build
admin-modal-new-build = NEW BUILD
admin-modal-new-build-sub = Version, Minecraft, and loader metadata
admin-modal-copy-from = Copy from
admin-modal-start-empty = Start empty
admin-modal-copy-hint = Carries over every file and setting. Nothing is re-uploaded.
admin-modal-build-version = Build version
admin-modal-optional-vanilla = Optional for vanilla

## Mod Catalog
admin-mods-title = MOD CATALOG
admin-mods-subtitle = Modrinth and CurseForge in one place
admin-mods-search-placeholder = Search mods…  ( / )
admin-mods-sort-relevance = Relevance
admin-mods-sort-downloads = Downloads
admin-mods-sort-follows = Followers
admin-mods-sort-updated = Recently updated
admin-mods-sort-newest = Newest
admin-mods-results-count = { $count } result(s)
admin-mods-results-none = No results yet
admin-mods-page = Page
admin-mods-source = Source
admin-mods-issues = Issues
admin-mods-tab-versions = Versions
admin-mods-tab-about = About
admin-mods-tab-gallery = Gallery
admin-mods-matching-only = Only versions matching this pack
admin-mods-no-versions = No matching versions.
admin-mods-no-screenshots = No screenshots.
admin-mods-downloads-count = downloads

## Capes
admin-capes-title = CAPES
admin-capes-subtitle = Cape Catalog & Cosmetics Management
admin-capes-add-title = Add New Cape
admin-capes-add-subtitle = Upload 64x32 or 22x17 Minecraft cape PNG textures.
admin-capes-name-label = Cape Name
admin-capes-name-placeholder = e.g. Mojang 2011, Cherry Blossom...
admin-capes-dropzone = Drop PNG file here or click to browse
admin-capes-dropzone-hint = PNG texture up to 512 KB
admin-capes-upload-btn = Upload Cape
admin-capes-uploading = Uploading…
admin-capes-grid-title = Cape Catalog Grid
admin-capes-empty-title = No capes in catalog
admin-capes-empty-text = Upload PNG cape textures to make them available for players to equip.

## Integrity
admin-integrity-title = INTEGRITY
admin-integrity-subtitle = What launchers found before starting the game
admin-integrity-note = Client-side signal, not proof: the launcher is open source and a patched build reports whatever it likes. Treat these as a reason to look, never as grounds for an automatic ban.
admin-integrity-unreviewed-only = Unreviewed only
admin-integrity-col-when = When
admin-integrity-col-finding = Finding
admin-integrity-col-subject = Subject
admin-integrity-col-build = Build
admin-integrity-col-player = Player
admin-integrity-repaired = repaired
admin-integrity-launcher = launcher
admin-integrity-open-card = open card
admin-integrity-reviewed = reviewed
admin-integrity-empty-title = Nothing flagged
admin-integrity-empty-text = Launchers verify mods and configs against the signed manifest before every launch.

## Blocklist
admin-blocklist-mask-placeholder = *xray*
admin-blocklist-sha1-placeholder = 40 hex chars
admin-blocklist-reason-placeholder = Known xray pack

## Rules
admin-rules-title = RULES
admin-rules-subtitle = The rulebook players read and moderators cite
admin-rules-search-placeholder = Search by code, title or wording
admin-rules-scope = Scope
admin-rules-scope-all = All scopes
admin-rules-public-page = Public page
admin-rules-btn-section = Section
admin-rules-btn-rule = Rule
admin-rules-empty-title = No rules yet
admin-rules-empty-text = Start with a section like “1 · Gameplay”, then add rules inside it.
admin-rules-other-section = Other rules
admin-rules-outside-section = outside any section
admin-rule-title-placeholder = Griefing another player's build
admin-rule-text-placeholder = What exactly counts as breaking this rule, and what does not
admin-sanc-min-placeholder = 30m · empty = no minimum
admin-sanc-max-placeholder = 7d · empty = forever allowed
admin-sanc-label-placeholder = first offence · repeated · in a rough form
admin-rule-delete-confirm = Delete rule { $code } “{ $title }”? Punishments that cite it keep their text.
admin-rule-delete-fail = Failed to delete rule
admin-rule-save-fail = Failed to save rule
admin-cat-delete-confirm = Delete section “{ $name }”? Its rules stay and move to “Other rules”.
admin-cat-delete-fail = Failed to delete section
admin-cat-save-fail = Failed to save section

## Wrapper
admin-wrapper-download-btn = Download wrapper.jar
admin-wrapper-subtitle = Every option the wrapper reads.
admin-wrapper-key = KEY
admin-wrapper-default = DEFAULT
admin-wrapper-meaning = MEANING
admin-wrapper-required = required

## Audit
admin-audit-title = AUDIT
admin-audit-subtitle = Who changed what, and when
admin-audit-event = Event
admin-audit-all-events = All events
admin-audit-target = Target
admin-audit-anything = Anything
admin-audit-target-id = Target id
admin-audit-optional = optional
admin-audit-uuid-placeholder = UUID
admin-audit-apply = Apply
admin-audit-reset = Reset
admin-audit-load-more = Load more
admin-audit-empty-title = Nothing recorded yet
admin-audit-empty-text = Logins, launches, integrity findings and every admin action land here as they happen.

## Settings
admin-settings-restart-title = Restart required
admin-settings-restart-desc = Settings are read once at startup. Nothing changes until the master restarts — and restarting drops launcher connections and interrupts downloads, so pick the moment yourself.
admin-settings-sec-general = General
admin-settings-sec-auth = Auth
admin-settings-sec-storage = Storage
admin-settings-sec-integrations = Integrations

## Server Wrapper
admin-wrapper-title = SERVER WRAPPER
admin-wrapper-lead = Agent installer and supervisor for game servers
admin-wrapper-setup-intro = ServerWrapper installs the right agent for your server, wires up authlib-injector, and keeps the server visible in the launcher while it boots. It is an installer and a supervisor — access control itself stays on the master.
admin-wrapper-setup-not-built = Not built yet — run ./gradlew collectAgents in agent/ and copy agent/build/agents/ into {NORO_DATA_DIR}/agents/.
admin-wrapper-setup-step2-desc = The agent secret is issued per game server in the build admin panel, section Game servers. It lives here and nowhere else — the wrapper passes it to the server process itself.
admin-wrapper-setup-step2-min = That is the minimum. NeoForge and Forge start from an args file, not a jar — pass it with a leading @, for example server-jar=@libraries/net/neoforged/neoforge/21.1.248/unix_args.txt, and keep their own JVM file as jvm-args=@user_jvm_args.txt. Every other option is listed below.
admin-wrapper-setup-step3-desc = The wrapper detects the platform and Minecraft version, installs the matching agent, verifies its signature, and launches the server.
admin-wrapper-setup-step4-desc = Sessions are validated against this master, so the server must stay in online mode.

admin-wrapper-opt-master-url = Master node address. A trailing slash is stripped.
admin-wrapper-opt-secret = Agent secret for this game server, issued in the build admin panel under Game servers. Must start with noroagent_. Lives only here — the wrapper hands it to the server process through an environment variable.
admin-wrapper-opt-server-jar = Server jar, relative to server-dir. Starts with @ for an args file instead — NeoForge and Forge launch that way: @libraries/net/neoforged/neoforge/21.1.248/unix_args.txt
admin-wrapper-opt-signing-public-key = Master ed25519 key, hex. Empty means fetch once and pin to noro/signing-key.pub; a later change becomes a hard error. Set it explicitly in production — a pinned key here is the real trust anchor.
admin-wrapper-opt-server-dir = Server directory. Everything else resolves against it, and the agent goes into its plugins/ or mods/.
admin-wrapper-opt-java = Java binary. Point it at a specific JDK when the default one is the wrong version for this Minecraft release.
admin-wrapper-opt-jvm-args = JVM arguments, split on whitespace. Accepts an @-file: NeoForge keeps its own as @user_jvm_args.txt.
admin-wrapper-opt-server-args = Arguments passed after the jar or args file. Set empty to pass none.
admin-wrapper-opt-platform = paper, fabric, neoforge or forge. Overrides detection — needed when several loader versions sit in libraries/ and the guess is ambiguous.
admin-wrapper-opt-mc-version = Minecraft version, e.g. 1.21.1. Overrides detection.
admin-wrapper-fb-fetch-pin = fetch and pin
admin-wrapper-fb-detected = detected

admin-agent-title = Agents
admin-agent-lead = The wrapper installs these itself — download one only to place it by hand.
admin-agent-not-built = Nothing built yet. Run ./gradlew collectAgents in agent/ and copy agent/build/agents/ into {NORO_DATA_DIR}/agents/.
admin-agent-versions-count = { $count } versions

































