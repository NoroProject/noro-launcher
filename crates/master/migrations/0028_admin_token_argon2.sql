-- Admin-токены: argon2 вместо SHA-256 без соли (INSTRUCTIONS §18.4).
--
-- Argon2 солёный, поэтому искать строку по нему нельзя. Разводим две роли,
-- которые раньше исполняло одно значение:
--
--   token_lookup — селектор, быстрый индексируемый SHA-256. Найти строку.
--   token_hash   — верификатор, argon2. Доказать владение секретом.
--
-- Смысл разделения: утечка БД больше не даёт рабочих токенов. Раньше в
-- token_hash лежало ровно то, с чем сравнивался предъявленный токен, — то есть
-- содержимое таблицы само по себе открывало админ-API.
ALTER TABLE admin_tokens ADD COLUMN IF NOT EXISTS token_lookup TEXT;

-- Существующие токены: их секрет мастеру неизвестен, пересчитать argon2 не из
-- чего. Старый SHA-256 переезжает в селектор и остаётся временным
-- доказательством, пока токеном не воспользуются: при первом же успешном
-- запросе мастер досчитает argon2 по предъявленному секрету. До тех пор
-- token_hash пуст — это и есть пометка «ещё на старой схеме».
ALTER TABLE admin_tokens ALTER COLUMN token_hash DROP NOT NULL;

UPDATE admin_tokens SET token_lookup = token_hash WHERE token_lookup IS NULL;
UPDATE admin_tokens SET token_hash = NULL WHERE token_hash = token_lookup;

ALTER TABLE admin_tokens ALTER COLUMN token_lookup SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS admin_tokens_lookup_idx ON admin_tokens (token_lookup);
