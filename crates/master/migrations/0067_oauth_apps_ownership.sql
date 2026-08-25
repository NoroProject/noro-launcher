-- Сторонние OAuth2-приложения: у приложения появляется владелец и модерация.
--
-- Раньше запись в `oauth_applications` мог завести только оператор руками в БД,
-- поэтому у приложения не было ни автора, ни состояния: всё, что лежало в
-- таблице, считалось разрешённым. Теперь приложение заводит игрок, а показывать
-- его чужим людям можно лишь после проверки — значит нужны и владелец, и статус.
--
-- `allowed_scopes` — потолок, а не запрос: приложение просит scope'ы у игрока,
-- но выйти за этот список не может. Базовые получают все, остальное выдаёт
-- оператор поимённо, чтобы «покажи мои привязки» не оказалось у первого
-- встречного счётчика онлайна.
--
-- `is_trusted` переименован в `is_official`: колонка ни на что не влияла, а
-- смысл у неё был именно такой — это наше собственное приложение. Официальные
-- не проходят модерацию и не гаснут общим выключателем сторонних.

ALTER TABLE oauth_applications RENAME COLUMN is_trusted TO is_official;

ALTER TABLE oauth_applications
    ADD COLUMN owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN status VARCHAR(16) NOT NULL DEFAULT 'approved',
    ADD COLUMN allowed_scopes TEXT NOT NULL DEFAULT 'identity profile skins capes punishments servers',
    ADD COLUMN review_note TEXT,
    ADD COLUMN reviewed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN reviewed_at TIMESTAMPTZ,
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Всё, что уже заведено руками, — наше и одобренное: другого способа попасть в
-- таблицу до этой миграции не было.
UPDATE oauth_applications SET status = 'approved';

-- Публичный клиент — приложение, которому секрет хранить негде: лаунчер стоит
-- на машине игрока, и всё, что в него зашито, считается известным. Признак был
-- строкой 'public' в поле хеша; теперь это колонка, а не соглашение.
ALTER TABLE oauth_applications
    ADD COLUMN is_public BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE oauth_applications SET is_public = TRUE WHERE client_secret_hash = 'public';

-- Адреса возврата хранились JSON-массивом в текстовом поле, а вводятся и
-- показываются построчно. Приводим к одному виду — разбирать оба формата в
-- коде значит держать эту развилку вечно.
UPDATE oauth_applications
   SET redirect_uris = 'http://127.0.0.1'
 WHERE redirect_uris LIKE '[%';

-- PKCE для публичных клиентов: код без секрета защищён только тем, что обменять
-- его может лишь тот, кто знает исходную строку challenge'а.
ALTER TABLE oauth_codes
    ADD COLUMN code_challenge TEXT,
    ADD COLUMN code_challenge_method VARCHAR(10);

COMMENT ON COLUMN oauth_applications.status IS 'pending | approved | rejected | suspended';

CREATE INDEX idx_oauth_apps_owner ON oauth_applications (owner_id);
CREATE INDEX idx_oauth_apps_status ON oauth_applications (status);

-- Сессия помнит, какому приложению её выдали.
--
-- Без этого «отозвать доступ» в кабинете значило лишь «больше не пускать»: сам
-- токен оставался живым до конца своих тридцати дней. Наши собственные входы
-- (сайт, лаунчер) остаются с пустым `app_id`.
ALTER TABLE oauth_sessions
    ADD COLUMN app_id UUID REFERENCES oauth_applications(id) ON DELETE CASCADE;

CREATE INDEX idx_oauth_sessions_app ON oauth_sessions (app_id);
