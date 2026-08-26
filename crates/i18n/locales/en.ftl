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
sidebar-no-identity = No platform linked

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
profile-skin-model = Model
profile-skin-model-classic = Classic
profile-skin-model-slim = Slim
notif-skin-model-failed = Could not change the skin model: { $reason }
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
notif-rate-limited = Too many attempts — try again in a minute
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
login-subtitle = SIGN IN
login-sign-in = SIGN IN
login-waiting = WAITING...
login-checking = CHECKING SESSION...
login-save-session = Save session
login-auto-login = Auto login
login-tagline = A Minecraft server launcher that installs mods and skins for you
login-sign-in-web = Sign in
login-web-hint = Opens the website — sign in there with any linked platform

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
mods-limited = BY PERMISSION
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
web-home-lead = A convenient Minecraft modded server project: one-click sign-in, automatic mod downloads, custom skins, and capes all in one place.
web-home-cta-cabinet = OPEN CABINET
web-home-cta-sign-in = GET STARTED
web-home-stat-online = Online
web-home-stat-servers = Servers
web-home-worlds-eyebrow = Worlds
web-home-worlds-title = Servers of the project
web-home-worlds-all = All servers
web-home-feature-builds-title = Fast Mod Setup
web-home-feature-builds-text = The launcher automatically downloads and updates all required mods in seconds — just click "Play".
web-home-feature-identity-title = Unified Account
web-home-feature-identity-text = One-click sign-in with Telegram, Discord, Twitch or Google — skins, capes and your cabinet all tied to one account.
web-home-feature-rules-title = Fair Play
web-home-feature-rules-text = Clear rules and transparent moderation — the rulebook is public and easy to navigate.
web-home-flow-eyebrow = Launch flow
web-home-flow-title = From sign-in to Minecraft in four steps
web-home-step-signin-title = Sign in
web-home-step-signin-text = Sign in on the website with any supported platform — your profile is instantly ready for game.
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
web-download-lead = Sign in, pick a server, and the launcher syncs the rest.
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
cabinet-passkeys-none-text = Add a Touch ID or Face ID key for fast sign-in without a platform
cabinet-launcher-title = Launcher
cabinet-roles-title = Roles — { $count }
cabinet-perms-count = { $count } perms
cabinet-roles-none-title = No roles yet
cabinet-roles-none-text = Server access is granted through roles.
cabinet-direct-perms-title = Direct permissions — { $count }
cabinet-direct-none-title = Nothing granted directly
cabinet-direct-none-text = That is normal — access usually comes from roles.

## Authorized applications
cabinet-apps-title = Applications
cabinet-apps-subtitle = Manage applications with account access
cabinet-apps-lead = Third-party services and launchers with access to your profile
cabinet-apps-refresh = Refresh
cabinet-apps-default-desc = Access to your Noro Network profile
cabinet-apps-revoke = Revoke Access
cabinet-apps-none-title = No third-party applications connected
cabinet-apps-none-text = Applications and launchers you have granted access to will appear here

## OAuth2 Authorization
oauth-loading-app = Loading application details…
oauth-official-app = Official application
oauth-requested-permissions = What it will get
oauth-wants-access = wants access to your account { $user }
oauth-by-developer = by { $owner }
oauth-app-pending = This application has not been reviewed yet, so only its author can sign in with it.
oauth-will-return-to = You will be sent back to { $host }
oauth-revoke-hint = You can revoke this access at any time in your cabinet.
oauth-cannot-continue = Cannot continue
oauth-back-home = Back to the site
oauth-missing-params = The application sent an incomplete request: client_id and redirect_uri are required.
oauth-scope-identity = Your nickname, UUID and avatar
oauth-scope-profile = Your roles and account status
oauth-scope-skins = Your skin and cape
oauth-scope-capes = Which capes you may wear
oauth-scope-punishments = Your active punishments
oauth-scope-servers = The project's server list and online count
oauth-scope-skins-write = Change your skin and cape
oauth-scope-identities = See which platforms you sign in with
oauth-scope-journal = Your launch history: builds and mods
oauth-scope-launcher = Full access to your account
oauth-deny = Cancel
oauth-allow = Authorize

## Skin and Capes Manager
skin-title = Skins & Capes
skin-subtitle = Modrinth & Pandora style skin manager
skin-3d-character = 3D Character
skin-custom-badge = Custom Skin
skin-default-badge = Default
skin-reset-default = Reset to Default
skin-model = Model
skin-model-classic = Classic
skin-model-slim = Slim
skin-model-hint = Slim arms are 3px wide instead of 4 — pick what the skin was drawn for
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
login-with = SIGN IN WITH { $provider }
login-no-methods = No sign-in method is configured on this instance yet. An operator has to turn one on in Admin → Sign-in.
login-passkey = SIGN IN WITH PASSKEY
login-recovery-toggle = Use a recovery code
login-username-placeholder = Username
login-submit = SIGN IN
login-recovery-hint = Each code works once.

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
admin-rule-punish-reason = Reason in a punishment
admin-rule-punish-reason-placeholder = Insulting other players
admin-rule-punish-reason-hint = What the punished player reads as the reason when a moderator names this rule and writes nothing. The rule title does not fit: «Respect other players» reads as praise, not as a reason.
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
admin-set-label-login_image_url = Login page image URL
admin-set-hint-login_image_url = Filled in automatically when you upload a file.
admin-set-hint-instance_name = Shown in the site header and as the main page heading.
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
admin-role-icon-hint = One character by the name.
admin-role-prefix-label = Prefix in game
admin-role-prefix-hint = Before the name. Colours with &.
admin-role-badge-preview = In-game badge
admin-role-badge-hint = Seen by players with the pack.
admin-role-badge-failed = Could not draw the badge
admin-role-badge-own = Custom image
admin-role-badge-pick = Upload
admin-role-badge-clear = Remove
admin-role-badge-uploaded = Image uploaded
admin-role-badge-cleared = Back to the drawn badge
admin-role-badge-rules = PNG, height a multiple of 7 (up to 56), width up to 12 heights. Replaces the whole badge.
admin-role-section-look = Appearance
admin-role-section-game = In game
admin-role-section-rights = Permissions and links
admin-role-saved = Role saved
admin-role-saved-hint = The badge updates after you sync in the role list.
admin-roles-sync = Sync
admin-roles-synced = Badges rebuilt
admin-roles-sync-hint = Rebuild the badges and hand them out to players.
admin-role-suffix-label = Suffix in game
admin-role-suffix-hint = After the name.
admin-role-inherits-label = Inherits from
admin-role-inherits-none = Nothing — own permissions only
admin-role-inherits-hint = The parent's permissions apply here.
admin-role-lp-label = LuckPerms group
admin-role-lp-hint = In-game group.
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
admin-tokens-perms-label = Permissions
admin-tokens-preset-title = Role Presets:
admin-tokens-preset-superadmin = Full Access
admin-tokens-preset-fulladmin = Admin Panel
admin-tokens-preset-senior-mod = Moderator
admin-tokens-preset-junior-mod = Helper
admin-tokens-preset-custom = Custom Set
admin-tokens-custom-toggle = Enter permissions manually as text
admin-tokens-checkbox-toggle = Pick with checkboxes
admin-tokens-select-all = Select all
admin-tokens-deselect-all = Deselect all

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
nav-admin-automod = AutoMod filters
nav-admin-moderation = Punishment texts
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
admin-gs-maintenance = Maintenance
admin-gs-maintenance-reason = Reason (optional)
admin-gs-maintenance-enable-all = Maintenance ON for all
admin-gs-maintenance-disable-all = Maintenance OFF for all
admin-gs-maintenance-countdown = Countdown before kick
admin-gs-maintenance-countdown-imm = Immediately (0s)
admin-gs-maintenance-countdown-30s = 30 seconds
admin-gs-maintenance-countdown-60s = 1 minute
admin-gs-maintenance-countdown-300s = 5 minutes
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
admin-news-search-placeholder = Search news by title or text
paging-range = { $from }–{ $to } of { $total }
paging-page = page { $page } of { $pages }
paging-empty = Nothing matches the search
admin-users-subtitle = Profiles, bans, roles, and direct permissions.
admin-users-player = Player
admin-users-identity = Platform
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
admin-users-identities = Sign-in methods
admin-users-identities-hint = Platforms this player signs in with. Unlink one when they lost access to it.
admin-users-identities-none = Nothing linked
admin-users-identities-none-hint = A local account created by staff: it has no platform behind it.
admin-users-identity-primary-hint = The account was created through this platform — its MC-UUID comes from here, so it cannot be unlinked.
admin-users-account-id = Account ID
admin-users-registered = Registered
admin-users-last-login = Last sign-in
admin-users-ban-reason = Ban reason
admin-users-flag-root = root
admin-users-flag-local = local account
admin-users-flag-no-play = cannot play
admin-users-flag-hidden = hidden from online
admin-users-flag-silent = silent join
admin-users-flag-frozen = frozen
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
admin-perm-no-permissions = No permissions granted yet.
admin-perm-show-nodes = Show all nodes
admin-perm-hide-nodes = Hide categories
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
# Счётчик рядом с числом: «3 sync rules».
admin-sync-rules-btn = sync rules
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
admin-settings-tab-branding = Branding
admin-settings-tab-general = General
admin-settings-tab-sign-in = Sign-in
admin-settings-tab-storage = Storage
admin-settings-tab-integrations = Integrations
admin-settings-tab-health = Health
admin-settings-logo-title = Logo
admin-settings-logo-desc = Site header icon. Empty falls back to the icon shipped with the build.
admin-settings-image-transparent = Transparent — no frame on the site
admin-settings-image-opaque = Opaque — framed on the site
admin-set-label-logo_url = Logo URL
admin-set-hint-logo_url = Filled in automatically when you upload a file.

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


































## Punishment texts
admin-moderation-title = Punishment texts
admin-moderation-subtitle = What a player sees when banned, muted or warned
admin-moderation-reset = Reset to default
admin-moderation-vars-title = Placeholders
admin-moderation-vars-lead = Anything else stays in the text as typed. Colours are written with &, like &c.
admin-moderation-var-player = Punished player
admin-moderation-var-reason = Reason given by the moderator
admin-moderation-var-duration = Time left, like 6d 23h
admin-moderation-var-expires = Date it runs out, UTC
admin-moderation-var-actor = Who issued it
admin-moderation-var-rule = Rule code, or a dash
admin-moderation-var-id = Case number, first eight characters
admin-moderation-var-kind = banned, muted or warned
admin-moderation-ban-perm = Network ban, forever
admin-moderation-ban-perm-hint = Disconnect screen. This is the whole conversation with the player — say where to appeal.
admin-moderation-ban-temp = Network ban, with a term
admin-moderation-ban-temp-hint = Same screen, but the player also needs to know when it runs out.
admin-moderation-sban-perm = Server ban, forever
admin-moderation-sban-perm-hint = Shown when the player is barred from this server only.
admin-moderation-sban-temp = Server ban, with a term
admin-moderation-sban-temp-hint = The player can still play elsewhere on the network.
admin-moderation-mute-perm = Mute, forever
admin-moderation-mute-perm-hint = The answer to every message the muted player tries to send.
admin-moderation-mute-temp = Mute, with a term
admin-moderation-mute-temp-hint = Same, with the time left. Silence reads as a broken server.
admin-moderation-warn = Warning
admin-moderation-warn-hint = Shown at once if the player is online, otherwise on their next login.
admin-moderation-broadcast = Announcement
admin-moderation-broadcast-hint = Shown to everyone on the server. Leave empty to punish quietly.
admin-moderation-receipt = Confirmation to the moderator
admin-moderation-receipt-hint = What the person who issued the punishment sees in reply.
admin-moderation-mute-bar-perm = Mute over the hotbar, forever
admin-moderation-mute-bar-temp = Mute over the hotbar, with a term
admin-moderation-mute-bar-hint = One line, no line breaks: the actionbar cuts off anything longer.
admin-moderation-warn-bar = Warning over the hotbar
admin-moderation-reason = Reason built from a rule
admin-moderation-reason-hint = Used when the moderator named a rule and wrote nothing. A rule with its own wording wins over this.
admin-moderation-no-account = No account screen
admin-moderation-no-account-hint = Shown to someone the master does not know — the real access boundary, reachable without the launcher
admin-moderation-no-access = No access screen
admin-moderation-no-access-hint = Shown when the account exists but has no access to this pack
admin-moderation-maintenance = Maintenance screen
admin-moderation-maintenance-hint = Shown while the server is under maintenance; holders of noro.server.maintenance.bypass still get in
admin-moderation-var-rule-title = Title of the rule
admin-moderation-var-rule-link = Clickable rule code

## Optional mods: why a mod cannot be turned on
optional-conflicts-with = Incompatible with { $mod }. Turn that one off first.
optional-needs-first = Turn on { $mod } first: this mod does not work without it.

## Reports & Freezes
admin-reports-title = Player Reports
admin-reports-subtitle = Report queue from game servers
admin-reports-open-only = Open only
admin-reports-resolve = Resolve Report
admin-reports-empty-title = No reports
admin-reports-empty-text = There are currently no open reports from players.
admin-freezes-section-title = Freeze / Unfreeze Player
admin-freezes-reason-label = Freeze Reason
admin-freezes-reason-placeholder = Cheat inspection / Software check
admin-freezes-action-freeze = Freeze
admin-freezes-action-unfreeze = Unfreeze
admin-freezes-frozen = Frozen
admin-freezes-active = Active
nav-admin-reports = Reports
cabinet-privacy-title = Online Privacy
cabinet-privacy-hide-online = Hide me from the public online list
cabinet-privacy-silent-join-title = Silent Join (Vanish on connect)
cabinet-privacy-silent-join-hint = Automatically connect to servers in vanish mode
cabinet-profile-title = Profile
cabinet-roles-card = Roles
cabinet-direct-card = Direct permissions
cabinet-activity-title = Game Activity (Heatmap)
cabinet-activity-days = { $count } days
cabinet-activity-minutes = { $count } min
cabinet-activity-hours = { $count } h
cabinet-activity-hour-short = h
cabinet-activity-total = this year
cabinet-activity-best = best day
cabinet-activity-none = no activity
cabinet-activity-less = less
cabinet-activity-more = more
cabinet-activity-empty-title = No activity data
cabinet-activity-empty-text = Played hours will appear after your first game sessions.

## In-game actions tab
admin-users-tab-game-actions = In-Game Actions
admin-game-actions-title = In-Game Direct Actions
admin-game-actions-subtitle = Send live commands for @{ $username } to active game server
admin-game-actions-status-online = Online on { $server }
admin-game-actions-status-offline = Offline
admin-game-actions-status-unknown = Status unknown — the master did not answer
admin-game-actions-offline-title = Player is currently offline
admin-game-actions-offline-text = In-game actions are disabled because @{ $username } is not connected to any live game server.
admin-game-actions-kick-title = Kick @{ $username }
admin-game-actions-kick-desc = Immediately disconnect player from the active game server.
admin-game-actions-kick-reason = Kick Reason
admin-game-actions-kick-placeholder = Kicked by administrator
admin-game-actions-kick-btn = Kick Player
admin-game-actions-tell-title = Direct Message (/tell)
admin-game-actions-tell-desc = Send a private in-game chat message to @{ $username }.
admin-game-actions-tell-msg = Message Content
admin-game-actions-tell-placeholder = Please check discord / support ticket
admin-game-actions-tell-btn = Send Message

## Server Broadcasts
admin-gs-announce-title = In-Game Server Broadcast
admin-gs-announce-target = Target Server
admin-gs-announce-all = All Servers of Build
admin-gs-announce-send = Send Broadcast
admin-gs-announce-placeholder = Type broadcast message for players...

## Mod Suggestions
admin-mod-suggestions-title = Requested Mods from Players ({ $count })
admin-mod-suggestions-expand = View Full List Modal
admin-mod-suggestions-search = Search by title, author, or description...
admin-mod-suggestions-open-external = Open mod page
admin-mod-suggestions-accept = Accept
admin-mod-suggestions-accept-optional = Add as Optional Mod
admin-mod-suggestions-accept-regular = Add as Regular Mod
admin-mod-suggestions-accept-servers = Regular + Install on Servers
admin-mod-suggestions-reject = Reject
admin-mod-suggestions-empty = No requested mods matching query

## Installed Build Mods
admin-installed-mods-title = Assembly Mods
admin-installed-mods-found = { $count } of { $total } mods found
admin-installed-mods-count = { $count } mods installed in build
admin-installed-mods-search = Search mods...
admin-installed-mods-grid = Grid view
admin-installed-mods-list = List view
admin-installed-mods-upload = Upload JAR
admin-installed-mods-catalog = Mod Catalog
admin-installed-mods-no-build = Select or create a build version above to manage installed mods.
admin-installed-mods-empty-title = No Mods Installed
admin-installed-mods-empty-text = Browse Modrinth catalog to add compatible mods or drag & drop custom jar files here.
admin-installed-mods-browse = Browse Modrinth Catalog
admin-installed-mods-no-match = No mods matching "{ $query }"




## Quick Search (Spotlight)
spotlight-quick-search = Quick Search...
spotlight-placeholder = Type a command, page, user, or server name... (Ctrl+K)
spotlight-no-results = No matching results found for "{ $query }"
spotlight-nav-instructions = Navigate with ↑ ↓
spotlight-open-instructions = Open with Cmd+K / Ctrl+K
spotlight-prefixes = @ people · # servers · > commands
spotlight-group-nav = Sections
spotlight-group-users = Players
spotlight-group-servers = Servers
spotlight-group-actions = Commands
spotlight-action-locale = Switch language to { $locale }
spotlight-action-logout = Sign out

## Search & Filters for User List
admin-users-search-label = Search User / UUID / platform
admin-users-search-placeholder = Filter by username, platform handle, or UUID...
admin-users-status-label = Status
admin-users-status-all = All Statuses
admin-users-status-active-only = Active Only
admin-users-status-banned-only = Banned Only
admin-users-role-label = Role
admin-users-role-all = All Roles



## AutoMod chat filters
admin-automod-title = AutoMod chat filters
admin-automod-subtitle = What the chat catches on its own, and what happens next
admin-automod-load-failed = Failed to load chat filters
admin-automod-enabled = ENABLED
admin-automod-disabled = DISABLED
admin-automod-mode = Action
admin-automod-mode-deny = Deny — block the message
admin-automod-mode-escalate = Escalate — warn first, mute on repeat
admin-automod-mode-punish = Punish — mute right away
admin-automod-mode-shadow = Shadow — let it through, notify staff
admin-automod-rule-code = Rule code
admin-automod-rule-code-hint = 1.4 or 2.2
admin-automod-whitelist = Allowed domains, comma separated
admin-automod-words = Forbidden words, comma separated
admin-automod-words-hint = word, another word
admin-automod-threshold = Caps ratio, 0.1 to 1.0
admin-automod-min-length = Minimum message length
admin-automod-max-messages = Messages allowed
admin-automod-window = Time window, seconds
admin-automod-ad-title = Ads
admin-automod-ad-hint = Links, domains and IP addresses in chat
admin-automod-word-title = Words
admin-automod-word-hint = Banned words and profanity
admin-automod-caps-title = Caps
admin-automod-caps-hint = Messages shouted in CAPITAL LETTERS
admin-automod-flood-title = Flood
admin-automod-flood-hint = Repeated messages sent in a burst
admin-automod-empty-title = No filters
admin-automod-empty-text = Chat filters have not been set up yet.

## Cases: report reviews
nav-admin-cases = Cases
admin-cases-title = Cases
admin-cases-subtitle = Player reports gathered into one review each
admin-cases-open-only = Open only
admin-cases-status = Status
admin-cases-status-open = Open
admin-cases-status-in_review = In review
admin-cases-status-resolved = Resolved
admin-cases-status-rejected = Dismissed
admin-cases-claimed-by = Handled by
admin-cases-reports = { $reports } reports from { $people } people
admin-cases-empty-title = No cases
admin-cases-empty-text = Nothing to review — reports open a case on their own.
admin-case-title = Case
admin-case-server = Server
admin-case-reports = Reports
admin-case-timeline = What happened
admin-case-chat = Chat around the event
admin-case-punish = Punish
admin-case-verdict = Verdict
admin-case-verdict-confirmed = Confirmed
admin-case-verdict-rejected = Not confirmed
admin-case-verdict-insufficient = Not enough evidence
admin-case-claim = Take the case
admin-case-release = Give it back
admin-case-probes = Ask the game
admin-case-probe-chat = Chat slice
admin-case-probe-inventory = Inventory
admin-case-probe-client = Client check
admin-case-note = Note
admin-case-note-hint = What you saw, in your own words
admin-case-note-add = Add note
admin-case-close = Close the case
admin-case-close-do = Close
admin-case-resolution-hint = What the reporter will be told
admin-case-reporter-stats = { $confirmed } of { $total } confirmed
admin-case-chat-forbidden = You have no permission to read the chat slice.
admin-case-chat-empty = No messages in this slice.
admin-case-chat-saved = { $count } messages attached
admin-case-channel-all = All
admin-case-channel-public = Global
admin-case-channel-local = Local
admin-case-channel-private = Direct
admin-case-channel-command = Commands
admin-case-event-report_added = Report filed
admin-case-event-claimed = Taken into work
admin-case-event-released = Given back
admin-case-event-teleport = Teleported
admin-case-event-freeze = Frozen
admin-case-event-watch_start = Started watching
admin-case-event-watch_stop = Stopped watching
admin-case-event-inventory_snapshot = Inventory captured
admin-case-event-chat_slice = Chat attached
admin-case-event-client_check = Client check asked
admin-case-event-screenshot = Screenshot attached
admin-case-event-note = Note
admin-case-event-punishment = Punishment issued
admin-case-event-verdict = Verdict
admin-case-missing-title = Case not found
admin-case-missing-text = It was removed, or the link is wrong.
admin-case-source-web = from the panel
admin-case-source-game = in game
admin-case-source-system = automatic
admin-case-opened = opened
admin-case-rule = rule
admin-case-report-no-place = place not recorded
admin-case-metric-reports = Reports
admin-case-metric-people = People
admin-case-metric-punishments = Punishments
admin-case-claim-hint = Locks the case on you and opens the review menu in game
admin-case-release-hint = Returns the case to the queue for anyone to take
admin-case-probe-chat-hint = Asks the server for the chat around the report
admin-case-probe-inventory-hint = Captures what the player carries right now
admin-case-probe-client-hint = Asks the player's launcher to verify the build
admin-case-verdict-confirmed-hint = The report holds up; punishment stands
admin-case-verdict-rejected-hint = Nothing happened; the reporter is told so
admin-case-verdict-insufficient-hint = No evidence either way; nobody is punished
admin-case-trust-title = Confirmed, dismissed and still open reports of this player
admin-blocklist-presets = Common cheats
admin-blocklist-preset-xray = Known X-Ray pack
admin-blocklist-preset-client = Hacked client
admin-blocklist-preset-bot = Pathfinding bot
admin-blocklist-preset-combat = Combat assistance
admin-blocklist-preset-freecam = Free camera
admin-blocklist-preset-macro = Autoclicker or macro
admin-blocklist-too-wide = That mask matches almost everything in the game folder
admin-blocklist-bad-hash = SHA1 is exactly 40 hex characters
admin-blocklist-sha1-mode-manual = by hand
admin-blocklist-sha1-mode-file = from a sample
admin-blocklist-sha1-pick = Pick the mod file
admin-blocklist-sha1-local = Hashed on your machine, the file stays.
admin-blocklist-sha1-no-crypto = Hashing needs a secure context (https or localhost)
admin-cases-search = Number, player or server
web-date-today = today
web-date-yesterday = yesterday
web-date-days = { $count } days ago
web-date-months = { $count } months ago
web-date-years = { $count } years ago
nav-admin-backup = Backup
admin-backup-title = Backup
admin-backup-subtitle = A copy of the master to fall back on
admin-backup-db = Database
admin-backup-db-text = Streams a pg_dump of the whole database: accounts, servers, rules, cases and punishments.
admin-backup-download = Download the dump
admin-backup-files-note = The plain dump holds the database only. Builds, skins, capes and agents live in the data volume — take the full archive to get them too.
admin-backup-full = Full archive
admin-backup-full-text = Database dump, every file in the data volume, a manifest and an Ed25519 signature, in one noro-backup-{ "{" }date{ "}" }.tar.gz. Built on the fly — about half a minute per 4 GB of data.
admin-backup-full-btn = Build the full archive
admin-backup-full-started = The browser is downloading the archive — watch its downloads list.
admin-backup-restore = Restore
admin-backup-restore-text = Upload an archive to inspect it. Nothing is touched until you confirm.
admin-backup-dropzone = Drop a noro-backup archive here, or click to pick one
admin-backup-dropzone-hint = .tar.gz built by this master
admin-backup-inspect-btn = Inspect the archive
admin-backup-uploading = uploading, { $percent }%
admin-backup-upload-failed = The upload did not reach the master
admin-backup-created = Built
admin-backup-master-version = Master version
admin-backup-schema = Schema version
admin-backup-schema-pair = archive { $archive }, master { $master }
admin-backup-size = Size
admin-backup-files = Files
admin-backup-signature = Signature
admin-backup-signature-ok = valid
admin-backup-signature-bad = does not match this master
admin-backup-unsigned = archive is unsigned
admin-backup-swap-note = Current files move to .noro-backup/old-{ "{" }date{ "}" } inside the data volume rather than being deleted, so a wrong archive can be undone by hand.
admin-backup-restore-btn = Restore from this archive
admin-backup-confirm-title = Restore the master?
admin-backup-confirm-text = This replaces the database and every file in the data volume. The master then restarts to come up on the restored data, and is unavailable for a few seconds.
admin-backup-confirm-btn = Yes, restore and restart
admin-backup-restore-done = Restored — the master is restarting

# Привязки платформ в кабинете
cabinet-identities-title = Linked platforms
cabinet-identities-lead = Sign in with any of them — they all lead to this account
cabinet-identities-none-title = No platforms linked
cabinet-identities-none-text = This account was created by an operator. Link a platform to sign in with it.
cabinet-identity-primary = Sign-up platform
cabinet-identity-nothing-to-link = This instance has no other sign-in methods set up yet.
cabinet-identity-unlink = Unlink
cabinet-identity-link = Link { $provider }
cabinet-identity-taken = That { $provider } account is already linked to another player.

# Админка: способы входа
admin-auth-title = SIGN-IN
admin-auth-subtitle = Which platforms players can sign in with
admin-auth-oauth-lead = OAuth app credentials from the provider portal
admin-auth-passkey-lead = Passwordless sign-in with Touch ID, Face ID or a security key
admin-auth-on = On
admin-auth-off = Off
admin-auth-not-configured = Turned on, but the app credentials are missing — the button will fail.
admin-auth-passkey-blocked = Needs the public site and API addresses over HTTPS.
admin-auth-secret-set = set
admin-auth-secret-unset = not set
admin-auth-secret-placeholder = Leave empty to keep the current one
admin-auth-redirect = Redirect URI for the provider portal
nav-group-auth = SIGN-IN
nav-group-profile = Profile
nav-group-account = Account

login-image-empty = No image set
admin-settings-login-image-title = Login page image
admin-settings-login-image-desc = Shown beside the sign-in form. GIF and WebP keep their animation and transparency.

login-heading = SIGN IN
login-lead = Pick a platform — signing in with any of them takes you to the same account.
login-recovery-back = Back to sign-in options
login-recovery-lead = Enter your username and one of your recovery codes.

## OAuth2 applications
app-status-pending = on review
app-status-approved = approved
app-status-rejected = rejected
app-status-suspended = blocked
cabinet-apps-connected = Connected applications
cabinet-apps-since = Access granted { $date }
cabinet-apps-revoke-confirm = Revoke access for "{ $name }"? Its tokens stop working right away.
cabinet-myapps-title = My applications
cabinet-myapps-lead = Applications you wrote yourself. A new one goes to review before other players can sign in with it.
cabinet-myapps-create = New application
cabinet-myapps-none-title = No applications yet
cabinet-myapps-none-text = Create one to let players sign in to your site or bot with their account here.
cabinet-myapps-name = Name
cabinet-myapps-name-hint = What players will see on the consent screen
cabinet-myapps-desc = Description
cabinet-myapps-desc-hint = One line about what the application does
cabinet-myapps-redirects = Redirect URIs
cabinet-myapps-redirects-hint = One per line. Players are sent back only to these addresses.
cabinet-myapps-scopes = Allowed
cabinet-myapps-pick-scopes = What the application will ask for
cabinet-myapps-pick-scopes-hint = Ask for the least you need: players see this list before they let you in. Anything beyond it is granted by the staff.
cabinet-myapps-edit = Edit
cabinet-myapps-cancel = Cancel
cabinet-myapps-edit-requeues = Saving sends the application back to review.
cabinet-myapps-delete-confirm = Delete "{ $name }"? Everyone who connected it loses access.
cabinet-myapps-rotate = Issue a new secret
cabinet-myapps-rotate-confirm = The old secret stops working immediately. Continue?
cabinet-myapps-secret-once = Client secret — shown once
cabinet-myapps-review-note = Reviewer
cabinet-myapps-icon-hint = Upload an icon: square, at least 64×64, up to 4 MB
admin-apps-title = Applications
admin-apps-subtitle = OAuth2 applications of this instance
admin-apps-toggles = Third-party applications
admin-apps-toggles-lead = Official applications are not affected by these switches.
admin-apps-enabled = Third-party applications work
admin-apps-enabled-hint = Off: nobody signs in with them and the section disappears from cabinets.
admin-apps-creation = Players may create applications
admin-apps-creation-hint = Off: existing ones keep working, new ones cannot be created.
admin-apps-filter-all = All
admin-apps-owner = Author
admin-apps-users = Players connected
admin-apps-scopes = What it may request
admin-apps-approve = Approve
admin-apps-reject = Reject
admin-apps-suspend = Block
admin-apps-delete = Delete
admin-apps-confirm = Confirm
admin-apps-note-placeholder = Reason — the author will see it
admin-apps-last-note = Last decision
admin-apps-delete-confirm = Delete "{ $name }"? Its tokens stop working right away.
admin-apps-none = No applications
admin-apps-none-hint = Applications created by players will show up here.
nav-admin-apps = Applications


# Permission Nodes
perm-group-panel = Панель (EN)
perm-group-players = Игроки (EN)
perm-group-moderation = Модерация (EN)
perm-group-rules = Правила (EN)
perm-group-servers = Серверы (EN)
perm-group-builds = Сборки (EN)
perm-group-machine = Машина (EN)
perm-group-other = Действия в игре (EN)
perm-group-content = Контент (EN)
perm-group-launcher = Лаунчер (EN)
perm-group-security = Защита (EN)
perm-group-support = Поддержка (EN)
perm-group-system = Система (EN)
perm-group-player = Игрок (EN)
perm-node-admin-stats = Метрики и статистика панели (EN)
perm-node-admin-agents = Список статусов агентов (EN)
perm-node-users-view = Просмотр списка игроков и профиля (EN)
perm-node-users-edit = Редактирование аккаунта и привязок (EN)
perm-node-users-username = Смена игрового ника (EN)
perm-node-users-delete = Удаление аккаунтов игроков (EN)
perm-node-users-roles = Назначение и снятие ролей (EN)
perm-node-users-permissions = Прямая выдача персональных прав (EN)
perm-node-users-notes-view = Просмотр заметок персонала (EN)
perm-node-users-notes-write = Создание заметок персонала (EN)
perm-node-users-notes-delete = Удаление заметок персонала (EN)
perm-node-users-sessions-view = Просмотр активных сессий (EN)
perm-node-users-sessions-kill = Завершение сессий игроков (EN)
perm-node-users-journal = Журнал запусков и событий (EN)
perm-node-users-launcher = Диагностика и удаленные действия (EN)
perm-node-users-skin = Смена скина игроку (EN)
perm-node-users-capes = Выдача плащей игроку (EN)
perm-node-impersonate = Вход под аккаунтом игрока (EN)
perm-node-punish-view = Просмотр истории наказаний (EN)
perm-node-punish-warn = Выдача предупреждений (варнов) (EN)
perm-node-punish-mute = Выдача блокировок чата (мутов) (EN)
perm-node-punish-ban = Выдача глобальных банов (EN)
perm-node-punish-server-ban = Бан на конкретном сервере (EN)
perm-node-punish-revoke = Снятие наказаний (EN)
perm-node-punish-bypass = Обход ограничений правил (EN)
perm-node-punish-permanent = Выдача перманентных наказаний (EN)
perm-node-freeze = Заморозка игроков (EN)
perm-node-reports-view = Просмотр жалоб игроков (EN)
perm-node-reports-resolve = Обработка жалоб игроков (EN)
perm-node-cases-view = Просмотр очереди и карточек дел (EN)
perm-node-cases-claim = Взятие дела в работу (EN)
perm-node-cases-resolve = Закрытие дела с вердиктом (EN)
perm-node-cases-chat = Просмотр среза чата в деле (EN)
perm-node-cases-inventory = Просмотр инвентаря игрока (EN)
perm-node-cases-watch = Слежка за игроком (EN)
perm-node-cases-client = Проверка лаунчера игрока (EN)
perm-node-rules-view = Просмотр свода правил (EN)
perm-node-rules-edit = Создание и редактирование правил (EN)
perm-node-rules-delete = Удаление правил и разделов (EN)
perm-node-servers-view = Просмотр серверов и сборок (EN)
perm-node-servers-edit = Создание и редактирование серверов (EN)
perm-node-servers-delete = Удаление серверов (EN)
perm-node-servers-agents = Игровые серверы и токены агентов (EN)
perm-node-servers-roles = Игровые роли сервера (EN)
perm-node-builds-view = Просмотр содержимого сборок (EN)
perm-node-builds-edit = Редактирование сборок и файлов (EN)
perm-node-builds-publish = Публикация сборок игрокам (EN)
perm-node-builds-delete = Удаление сборок (EN)
perm-node-builds-import = Импорт сборки модов (EN)
perm-node-mods-view = Просмотр каталога модов (EN)
perm-node-mods-install = Установка модов в сборку (EN)
perm-node-mods-remove = Удаление модов из сборки (EN)
perm-node-cores-edit = Ядра серверов и загрузчики (EN)
perm-node-wrapper-view = Просмотр состояния игровой машины (EN)
perm-node-wrapper-console = Чтение консоли сервера (EN)
perm-node-wrapper-command = Отправка команд в консоль (EN)
perm-node-wrapper-files = Чтение и запись файлов сервера (EN)
perm-node-wrapper-power = Управление питанием и перезапуск (EN)
perm-node-wrapper-backups = Создание и восстановление бэкапов (EN)
perm-node-game-kick = Кик игрока из игры (EN)
perm-node-game-tell = Отправка ЛС игроку в игре (EN)
perm-node-game-announce = Объявление в игре (EN)
perm-node-news-view = Просмотр новостей (EN)
perm-node-news-edit = Создание и публикация новостей (EN)
perm-node-news-delete = Удаление новостей (EN)
perm-node-translations-view = Просмотр переводов интерфейса (EN)
perm-node-translations-edit = Редактирование переводов интерфейса (EN)
perm-node-capes-view = Просмотр плащей (EN)
perm-node-capes-edit = Загрузка и удаление плащей (EN)
perm-node-launcher-view = Просмотр релизов лаунчера (EN)
perm-node-launcher-publish = Публикация релиза лаунчера (EN)
perm-node-launcher-deploy = Выкатка релизов игрокам (EN)
perm-node-launcher-clients = Клиенты лаунчера и их статус (EN)
perm-node-launcher-tokens = Выдача API-токенов лаунчера (EN)
perm-node-integrity-view = Просмотр флагов целостности (EN)
perm-node-integrity-review = Разбор флагов целостности (EN)
perm-node-blocklist-view = Просмотр черного списка файлов (EN)
perm-node-blocklist-edit = Редактирование черного списка (EN)
perm-node-chat-filters-view = Просмотр фильтров чата (EN)
perm-node-chat-filters-edit = Настройка автомодерации и фильтров (EN)
perm-node-chat-filters-delete = Удаление фильтров чата (EN)
perm-node-support-logs = Просмотр бандлов логов (EN)
perm-node-support-request = Запрос логов у игрока (EN)
perm-node-support-force = Принудительный сбор логов (EN)
perm-node-support-download = Скачивание бандлов логов (EN)
perm-node-support-delete = Удаление логов поддержки (EN)
perm-node-moderation-view = Просмотр модераторских шаблонов (EN)
perm-node-moderation-edit = Редактирование модераторских шаблонов (EN)
perm-node-roles-view = Просмотр ролей (EN)
perm-node-roles-edit = Создание и редактирование ролей (EN)
perm-node-audit = Просмотр журнала аудита (EN)
perm-node-settings-view = Просмотр настроек инстанса (EN)
perm-node-settings-edit = Изменение настроек инстанса (EN)
perm-node-auth-methods-view = Просмотр способов входа (EN)
perm-node-auth-methods-edit = Настройка способов входа (EN)
perm-node-tokens-view = Просмотр API-токенов панели (EN)
perm-node-tokens-manage = Создание и отзыв API-токенов панели (EN)
perm-node-restarts-view = Просмотр расписания перезапусков (EN)
perm-node-restarts-edit = Управление расписанием перезапусков (EN)
perm-node-oauth-view = Просмотр OAuth2-приложений (EN)
perm-node-oauth-manage = Модерация OAuth2-приложений (EN)
perm-node-storage = Очистка неиспользуемых объектов стора (EN)
perm-node-backup = Скачивание дампа базы данных (EN)
perm-node-backup-restore = Восстановление мастера из архива (EN)
perm-node-launcher-beta = Бета-канал лаунчера (EN)
perm-group-access = Access
perm-group-game = Server
perm-group-branches = Branches
perm-superadmin-desc = Superadmin: Full unlimited access
perm-admin-all-desc = Panel Admin: All administration sections
