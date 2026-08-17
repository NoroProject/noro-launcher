-- Принудительное обновление англоязычных переводов всех категорий и правил

-- 1. Переводы разделов
INSERT INTO rule_category_translations (category_id, locale, name, description)
SELECT c.id, 'en', t.name, t.description
FROM rule_categories c
JOIN (VALUES 
    ('1', 'General Regulations', 'Rules apply across all project servers, launcher and Discord. Ignorance of rules does not exempt from punishment.'),
    ('2', 'Chat & Communication', 'Everything a player writes in game chat, direct messages and voice channels.'),
    ('3', 'Gameplay', 'Fair play: what is allowed and what is forbidden using server mechanics.'),
    ('4', 'Client & Modifications', 'What is permitted in the client and what is considered a cheat.'),
    ('4.1', 'Allowed Modifications', 'Client mods that do not provide an unfair advantage and are not banned.'),
    ('5', 'Builds & Claims', 'Griefing, theft and private property.'),
    ('6', 'Account & Security', 'Responsibility for your account and prohibition of ban evasion.'),
    ('7', 'Staff & Moderation', 'Communication with staff and handling appeals.')
) AS t(code, name, description) ON lower(c.code) = lower(t.code)
ON CONFLICT (category_id, locale) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description;

-- 2. Переводы правил (все 30 пунктов)
INSERT INTO rule_translations (rule_id, locale, title, description)
SELECT r.id, 'en', t.title, t.description
FROM rules r
JOIN (VALUES
    ('1.1', 'Respect for Other Players', 'Insults, harassment, threats and any attempts to ruin another person''s game are forbidden. This applies to jokes if requested to stop.'),
    ('1.2', 'Hate Speech & Discrimination', 'Statements disparaging a person based on nationality, religion, gender or orientation. Instant penalty with no debate.'),
    ('1.3', 'Illegal & Explicit Content', 'Drugs, weapons, extremism, pornography and anything violating the law. Sharing links to such materials is treated as publishing them.'),
    ('1.4', 'Third-Party Advertising', 'Invitations to other servers, Discord communities and commercial sales outside the project. Sending links to friends in DMs is exempt.'),
    ('2.1', 'Flood & Spam', 'Repeated messages, random characters, caps lock in global chat. Single accidental repetition is fine; a series is a violation.'),
    ('2.2', 'Profanity in Global Chat', 'Explicit/profane language in global chat and usernames. Allowed in private messages if all participants consent.'),
    ('2.3', 'Baiting & Trolling', 'Intentional attempts to provoke or upset others, including bypassing chat filters.'),
    ('2.4', 'False Staff Reports', 'Fabricated complaints, false moderator calls and spamming support with joke tickets.'),
    ('2.5', 'Staff Impersonation', 'Usernames, skins or messages claiming or implying the player is a moderator or administrator.'),
    ('3.1', 'Bug & Glitch Exploiting', 'Item duping, passing through blocks, claim bypass via plugin glitches. Discovered bugs must be reported to staff immediately.'),
    ('3.2', 'Server Lag Machines', 'Mechanisms, farms and builds intentionally created to cause lag: infinite redstone clocks, thousands of entities in one chunk.'),
    ('3.3', 'AFK Farming & Bypasses', 'Autoclickers, macros and key weights used to spoof active player status.'),
    ('3.4', 'Spawn Camping & Newbie Killing', 'PvP in peaceful zones or hunting players who just joined the server and cannot retaliate.'),
    ('3.5', 'Player Traps', 'Hidden lava pits, dead-end portals and structures designed solely to kill incoming players.'),
    ('4.1', 'Cheats & Unauthorized Software', 'X-Ray, killaura, fly, freecam, auto-totem and any mods providing an unfair advantage. Client integrity checks detect such mods automatically.'),
    ('4.1.1', 'Optimization & UI Mods', 'Sodium, Iris, minimaps without entity/player radar, JEI, inventory tweaks — allowed.'),
    ('4.1.2', 'Minimap with Player Radar', 'Displaying enemy positions on minimaps grants an unfair PvP advantage and is banned, even if the mod is otherwise allowed.'),
    ('4.2', 'X-Ray Resource Packs', 'Texture packs making stone or ore transparent are treated as X-Ray regardless of name.'),
    ('4.3', 'Integrity Check Bypasses', 'Modifying manifests, launching the game outside the launcher with modified builds, or attempting to disable integrity checks.'),
    ('5.1', 'Griefing', 'Destructing or modifying another player''s builds without owner consent.'),
    ('5.2', 'Theft', 'Taking items from another player''s chests, farms or mechanisms. Clan warehouses follow clan rules.'),
    ('5.3', 'Encroaching Builds', 'Building right up against another player''s land to block view or access. Disputes resolved by staff based on build date.'),
    ('5.4', 'Offensive Builds & Signs', 'Builds, redstone art and signs featuring insults, hate speech symbols or pornography.'),
    ('6.1', 'Account Responsibility', 'Anything done from your account is treated as done by you. Sharing credentials with others is at your own risk.'),
    ('6.2', 'Ban Evasion', 'Logging in from another account during an active ban. The original ban duration restarts and the alt account is permanently banned.'),
    ('6.3', 'Scams & Fraud', 'Deception during trade, fake giveaways, collecting money for "a spot on the team".'),
    ('6.4', 'Doxxing & Privacy Violation', 'Real name, address, phone number or private chat leaks without consent. This penalty is non-appealable.'),
    ('7.1', 'Disobeying Staff', 'Ignoring direct, reasonable requests from moderators acting within the rules.'),
    ('7.2', 'Begging', 'Repeatedly asking staff or players for items, currency, privileges or moderator status.'),
    ('7.3', 'Punishment Appeals', 'Appeals are handled in the cabinet or Discord using the rule code. Arguing in global chat does not lift a punishment.')
) AS t(code, title, description) ON lower(r.code) = lower(t.code)
ON CONFLICT (rule_id, locale) DO UPDATE SET title = EXCLUDED.title, description = EXCLUDED.description;

-- 3. Переводы вариантов наказаний
INSERT INTO rule_sanction_translations (sanction_id, locale, label)
SELECT s.id, 'en', t.label
FROM rule_sanctions s
JOIN (VALUES
    ('первое нарушение', 'first offense'),
    ('повторное', 'repeated offense'),
    ('массовый дюп', 'mass dupe'),
    ('без наживы', 'no profit'),
    ('после предупреждения', 'after warning'),
    ('предупреждение', 'warning')
) AS t(ru_label, label) ON s.label = t.ru_label
ON CONFLICT (sanction_id, locale) DO UPDATE SET label = EXCLUDED.label;
