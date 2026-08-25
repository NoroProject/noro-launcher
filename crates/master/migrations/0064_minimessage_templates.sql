-- Шаблоны наказаний на MiniMessage.
--
-- Раньше цвет в них задавался как `#rrggbb`, а начертание — legacy-кодами вроде
-- `&l`. Теперь разметка одна на всё: `<#rrggbb>` и `<bold>`. Переписываем то,
-- что администратор успел сохранить: иначе после обновления он увидел бы в
-- экране бана голые решётки вместо цвета.
--
-- Цвет заворачивается только там, где перед ним не стоит угловая скобка, —
-- миграцию могут прогнать повторно на восстановленной копии, и дважды
-- обёрнутый `<<#e6e6e6>>` сломал бы разбор. Lookbehind в Postgres нет, поэтому
-- предыдущий символ захватывается в группу и возвращается на место.
UPDATE instance_settings
SET value = (
    SELECT jsonb_object_agg(lang, (
        SELECT jsonb_object_agg(
            key,
            to_jsonb(
                replace(
                    replace(
                        replace(
                            replace(
                                replace(
                                    regexp_replace(
                                        text #>> '{}',
                                        '(^|[^<])#([0-9a-fA-F]{6})',
                                        '\1<#\2>',
                                        'g'
                                    ),
                                    '&l', '<bold>'
                                ),
                                '&o', '<italic>'
                            ),
                            '&n', '<underlined>'
                        ),
                        '&m', '<strikethrough>'
                    ),
                    '&r', '<reset>'
                )
            )
        )
        FROM jsonb_each(templates) AS t(key, text)
        WHERE jsonb_typeof(text) = 'string'
    ))
    FROM jsonb_each(value) AS m(lang, templates)
)
WHERE key = 'moderation_messages';
