-- Файл рукопожатия лаунчера с клиентским модом разбора не принадлежит сборке.
--
-- `noro-bridge.json` кладёт лаунчер на запуск игры и убирает на выходе: там
-- порт и одноразовый ключ, и в манифесте его быть не может. Сверка сегодня
-- обходит только `mods/` и `config/`, так что корневой файл она и не увидит, —
-- но полагаться на границу обхода вместо явного правила значит однажды
-- получить `ForbiddenOptionalMod` на собственный файл лаунчера.

ALTER TABLE builds
ALTER COLUMN unmanaged_paths SET DEFAULT '["saves/","screenshots/","options.txt","optionsof.txt","optionsshaders.txt","logs/","crash-reports/","xaero*","config/xaero*","xaerominimap*","xaeroworldmap*","noro-bridge.json"]'::jsonb;

UPDATE builds
SET unmanaged_paths = unmanaged_paths || '["noro-bridge.json"]'::jsonb
WHERE NOT (unmanaged_paths @> '["noro-bridge.json"]'::jsonb);
