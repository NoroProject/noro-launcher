-- Разрешать ли игрокам предлагать моды для сборки
ALTER TABLE builds ADD COLUMN IF NOT EXISTS allow_optional_mod_suggestions BOOLEAN NOT NULL DEFAULT TRUE;
