-- Журнал админских действий. Раньше не было ничего: кто выдал право, кто снёс
-- сборку и когда именно сменилась роль — восстанавливалось только по памяти.
--
-- Хранится бессрочно (строки лёгкие). Смысл журнала в том, чтобы через полгода
-- можно было разобрать спорную ситуацию, а retention это ровно и ломает.
CREATE TABLE IF NOT EXISTS audit_log (
    id          BIGSERIAL PRIMARY KEY,
    at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Кто. actor_id зануляется вместе с удалением пользователя, поэтому рядом
    -- лежит подпись строкой: запись должна пережить удаление аккаунта, иначе
    -- «кто это сделал» теряется ровно в том случае, когда важнее всего.
    actor_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_label TEXT NOT NULL,

    -- Что: "user.role.add", "build.publish", "admin.request".
    action      TEXT NOT NULL,
    -- Над чем: ("user", "<uuid>"), ("build", "<uuid>").
    target_kind TEXT,
    target_id   TEXT,

    -- Подробности события: diff изменения, метод и путь запроса, причина.
    details     JSONB NOT NULL DEFAULT '{}'::jsonb,
    ip          TEXT
);

CREATE INDEX IF NOT EXISTS audit_log_at_idx ON audit_log (at DESC);
CREATE INDEX IF NOT EXISTS audit_log_actor_idx ON audit_log (actor_id, at DESC);
CREATE INDEX IF NOT EXISTS audit_log_target_idx ON audit_log (target_kind, target_id, at DESC);
CREATE INDEX IF NOT EXISTS audit_log_action_idx ON audit_log (action, at DESC);
