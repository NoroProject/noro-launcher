-- Дополнительные свойства ролей.
--
-- lp_group — имя группы в LuckPerms. Базы намеренно раздельные: у мастера своя
-- модель ролей, у LuckPerms своя. Это поле — единственная точка их связи, по
-- нему синхронизатор поймёт, какой группе соответствует роль. NULL означает,
-- что роль в игру не проецируется.
--
-- icon — короткая строка: имя иконки из набора либо юникод-символ. Хранится
-- текстом, а не файлом: роль показывается рядом с ником, и там нужен глиф,
-- переживающий и веб, и чат в игре.

ALTER TABLE roles ADD COLUMN IF NOT EXISTS lp_group TEXT;
ALTER TABLE roles ADD COLUMN IF NOT EXISTS icon TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_roles_lp_group
    ON roles (lp_group) WHERE lp_group IS NOT NULL;
