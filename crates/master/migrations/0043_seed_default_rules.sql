-- Переводы категорий и правил по коду
INSERT INTO rule_category_translations (category_id, locale, name, description)
SELECT c.id, 'en', t.name, t.description
FROM rule_categories c
JOIN (VALUES 
    ('1', 'Gameplay', 'Rules regarding ingame behavior and interaction with players'),
    ('2', 'Chat & Communication', 'Rules regarding global, local and private chat communication'),
    ('3', 'Accounts & Security', 'Rules regarding account security and third-party software')
) AS t(code, name, description) ON lower(c.code) = lower(t.code)
ON CONFLICT (category_id, locale) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description;

INSERT INTO rule_translations (rule_id, locale, title, description)
SELECT r.id, 'en', t.title, t.description
FROM rules r
JOIN (VALUES
    ('1.1', 'Cheats & Hacked Clients', 'Use of cheat clients, X-Ray mods, autoclickers, scripts or any unauthorized modifications providing an unfair advantage.'),
    ('1.2', 'Griefing & Property Damage', 'Destruction or alteration of another player''s builds, spilling lava/water, traps around claims or intentional landscape damage.'),
    ('1.3', 'Bug Exploiting', 'Exploiting game, server or mod bugs for item dupe, claim bypass or unfair gain. Found bugs must be reported to administration.'),
    ('1.4', 'Gameplay Interference', 'Intentionally disrupting events, creating lag machines, spamming moderators or filing false reports.'),
    ('2.1', 'Insults & Toxicity', 'Insulting other players, baiting/provoking, excessive toxicity or inappropriate language in chat.'),
    ('2.2', 'Family Insults', 'Any derogatory mention or insult directed towards family members or relatives of other players.'),
    ('2.3', 'Spam, Flood & Caps Lock', 'Sending repeating messages, meaningless symbols, command flooding or abusing CAPS LOCK in chat.'),
    ('2.4', 'Advertising & RMT', 'Promoting third-party servers, Discord servers, commercial sites or selling ingame items for real money (RMT).'),
    ('3.1', 'Account Sharing & Sale', 'Sharing account credentials with third parties, buying or selling accounts. Owners are responsible for actions under their account.'),
    ('3.2', 'Ban Evasion & Alt Accounts', 'Creating alt accounts to bypass active bans or mutes on servers.'),
    ('3.3', 'Staff Impersonation', 'Using usernames, prefixes or skins imitating project administration, or misleading players on behalf of staff.')
) AS t(code, title, description) ON lower(r.code) = lower(t.code) AND r.server_id IS NULL
ON CONFLICT (rule_id, locale) DO UPDATE SET title = EXCLUDED.title, description = EXCLUDED.description;

-- Фолбэк переводов всех остальных правил на английский язык
INSERT INTO rule_category_translations (category_id, locale, name, description)
SELECT id, 'en', name, description FROM rule_categories
ON CONFLICT (category_id, locale) DO NOTHING;

INSERT INTO rule_translations (rule_id, locale, title, description)
SELECT id, 'en', title, description FROM rules
ON CONFLICT (rule_id, locale) DO NOTHING;

INSERT INTO rule_sanction_translations (sanction_id, locale, label)
SELECT id, 'en', label FROM rule_sanctions
ON CONFLICT (sanction_id, locale) DO NOTHING;
