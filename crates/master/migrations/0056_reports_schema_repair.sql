-- Выправление схемы репортов и заморозок.
--
-- На базах, которые прошли через промежуточные версии волны 3, схема разошлась
-- с тем, что читает код: осталась таблица `reports` от первой редакции и
-- колонка `player_freezes.unfreezed_at` от переименования в `released_at`,
-- а `player_reports` и `pending_report_feedbacks` не появились вовсе —
-- админка падала с «relation "player_reports" does not exist».
--
-- Свежая база сюда доходит уже правильной: всё ниже идемпотентно и на ней
-- просто ничего не делает. Смысл файла в том, чтобы разошедшиеся базы
-- догнали её, а не в том, чтобы что-то создать заново.

-- Те же определения, что в 0050. Повторены дословно: база, где 0050 по любой
-- причине не оставила таблиц, иначе останется без них навсегда — миграция
-- второй раз не запускается.
CREATE TABLE IF NOT EXISTS player_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    game_server_id UUID NOT NULL REFERENCES game_servers(id) ON DELETE CASCADE,
    reason TEXT NOT NULL,
    world TEXT,
    x DOUBLE PRECISION,
    y DOUBLE PRECISION,
    z DOUBLE PRECISION,
    status TEXT NOT NULL DEFAULT 'open',
    claimed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    claimed_at TIMESTAMPTZ,
    resolution TEXT,
    punishment_id UUID REFERENCES punishments(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_player_reports_status ON player_reports(status);
CREATE INDEX IF NOT EXISTS idx_player_reports_created ON player_reports(created_at DESC);

CREATE TABLE IF NOT EXISTS pending_report_feedbacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_username TEXT NOT NULL,
    resolution TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_pending_report_feedbacks_user ON pending_report_feedbacks(user_id);

-- Наследство переименования. Колонка всегда была NULL-able и её никто не
-- заполнял: снятие заморозки пишется в `released_at`.
ALTER TABLE player_freezes DROP COLUMN IF EXISTS unfreezed_at;

-- Первая редакция таблицы репортов. Кода, который бы в неё писал, не
-- существовало ни дня, поэтому она пуста — но удаляем только пустую: молча
-- терять строки из-за предположения нельзя.
--
-- Проверка и удаление — через EXECUTE и вложенным IF, а не одним условием:
-- PL/pgSQL планирует выражение целиком, поэтому упоминание отсутствующей
-- таблицы роняет блок ещё до того, как сработает короткое замыкание `AND`.
-- На свежей базе это как раз тот случай.
DO $$
DECLARE
    leftover BIGINT;
BEGIN
    IF to_regclass('public.reports') IS NOT NULL THEN
        EXECUTE 'SELECT count(*) FROM reports' INTO leftover;
        IF leftover = 0 THEN
            EXECUTE 'DROP TABLE reports';
        ELSE
            RAISE WARNING 'таблица reports не пуста (% строк), оставляю как есть', leftover;
        END IF;
    END IF;
END $$;
