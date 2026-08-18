-- Модель скина: классическая (Стив) или тонкая (Алекс).
--
-- До сих пор мастер отдавал текстуру без метаданных, а это для клиента значит
-- «classic»: у всех, кто нарисовал скин под Алекс, в игре толстые руки, и
-- поправить это игрок ничем не мог.
--
-- Хранится флагом, а не строкой: моделей ровно две, и `model=slim` — то
-- единственное, что понимает Yggdrasil. Пресеты помнят свою модель отдельно:
-- иначе переключение между сохранёнными скинами ломало бы вид.

ALTER TABLE users ADD COLUMN IF NOT EXISTS skin_slim BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE user_skin_presets ADD COLUMN IF NOT EXISTS skin_slim BOOLEAN NOT NULL DEFAULT FALSE;
