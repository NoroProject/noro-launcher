-- Fluent (.ftl) catalogues, kept inline instead of in the file store. They are a
-- few KB and always replaced whole, so content addressing would only buy us a
-- garbage collection pass on every save from the admin panel.

CREATE TABLE IF NOT EXISTS translations (
    locale     TEXT PRIMARY KEY,
    ftl        TEXT NOT NULL,
    sha1       TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID REFERENCES users(id)
);
