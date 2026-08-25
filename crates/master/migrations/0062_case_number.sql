-- Человеческий номер дела.
--
-- UUID в разборе не назовёшь ни в игре, ни в апелляции: его не продиктовать и
-- не запомнить. Номер идёт подряд и печатается как `N-000000001` — по нему
-- игрок ссылается на своё дело, а модератор ищет его в очереди.
--
-- Отдельная колонка, а не префикс UUID: подряд идущие номера ещё и показывают,
-- сколько разборов было всего.

ALTER TABLE cases ADD COLUMN IF NOT EXISTS number BIGINT;

CREATE SEQUENCE IF NOT EXISTS cases_number_seq OWNED BY cases.number;

-- Уже заведённым делам номера раздаются по времени открытия: порядок номеров
-- должен совпадать с порядком разборов, иначе он ничего не значит.
UPDATE cases SET number = ordered.row_number
  FROM (
    SELECT id, row_number() OVER (ORDER BY opened_at) AS row_number FROM cases WHERE number IS NULL
  ) AS ordered
 WHERE cases.id = ordered.id AND cases.number IS NULL;

SELECT setval('cases_number_seq', COALESCE((SELECT MAX(number) FROM cases), 0) + 1, false);

ALTER TABLE cases ALTER COLUMN number SET DEFAULT nextval('cases_number_seq');
ALTER TABLE cases ALTER COLUMN number SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_cases_number ON cases (number);
