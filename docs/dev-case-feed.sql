-- Лента разбора и срез чата тестового дела.
--
-- Времена заданы от «сейчас» и идут по порядку разбора: жалобы, взятие в
-- работу, срез чата, телепорт, наблюдение, инвентарь, проверка клиента,
-- заметка, наказание. Ровно то, что модератор увидит в живом деле.

BEGIN;

DELETE FROM case_events   WHERE case_id = 'c0000000-0000-4000-8000-000000000001';
DELETE FROM case_messages WHERE case_id = 'c0000000-0000-4000-8000-000000000001';

INSERT INTO case_events (case_id, at, actor_id, actor_label, source, kind, payload)
SELECT 'c0000000-0000-4000-8000-000000000001', NOW() - v.ago, v.actor, v.label, v.source, v.kind, v.payload::jsonb
  FROM (VALUES
    (INTERVAL '38 minutes', 'd0000000-0000-4000-8000-000000000002'::uuid, 'AlexBuilder', 'game', 'report_added',
     '{"reason":"Снёс половину моего дома и залил лавой, пока я был в шахте"}'),
    (INTERVAL '31 minutes', 'd0000000-0000-4000-8000-000000000003'::uuid, 'MinerJoe', 'game', 'report_added',
     '{"reason":"Он же вынес мои сундуки на общем складе клана"}'),
    (INTERVAL '19 minutes', 'd0000000-0000-4000-8000-000000000002'::uuid, 'AlexBuilder', 'game', 'report_added',
     '{"reason":"Продолжает ломать, я не успеваю чинить"}'),
    (INTERVAL '14 minutes', NULL, 'Dalynkaa', 'web', 'claimed', '{}'),
    (INTERVAL '14 minutes', NULL, '', 'system', 'chat_slice', '{"messages":14}'),
    (INTERVAL '13 minutes', NULL, 'Dalynkaa', 'game', 'teleport', '{"to":"place"}'),
    (INTERVAL '12 minutes', NULL, 'Dalynkaa', 'game', 'watch_start', '{}'),
    (INTERVAL '11 minutes', NULL, 'Dalynkaa', 'game', 'inventory_snapshot',
     '{"items":["Алмазная кирка x1","Ведро лавы x6","Динамит x24","Дубовые доски x384","Алмаз x57"]}'),
    (INTERVAL '10 minutes', NULL, 'Dalynkaa', 'game', 'watch_stop', '{}'),
    (INTERVAL '10 minutes', NULL, 'Dalynkaa', 'web', 'client_check', '{"target":"GrieferSteve"}'),
    (INTERVAL '9 minutes',  NULL, 'Dalynkaa', 'web', 'punishment',
     '{"punishment_id":"f0000000-0000-4000-8000-000000000001","kind":"mute","reason":"Оскорбление участников","rule":"1.1"}'),
    (INTERVAL '8 minutes',  NULL, 'Dalynkaa', 'game', 'freeze', '{"target":"GrieferSteve"}'),
    (INTERVAL '6 minutes',  NULL, 'Dalynkaa', 'web', 'note',
     '{"text":"Ведро лавы и динамит в инвентаре, рядом с домом AlexBuilder следы поджога. Проверка клиента чистая — это не чит, а гриферство. Жду ответа игрока перед баном."}')
  ) AS v(ago, actor, label, source, kind, payload);

-- Автор события — Dalynkaa там, где он не задан явно.
UPDATE case_events
   SET actor_id = (SELECT id FROM users WHERE mc_username = 'Dalynkaa')
 WHERE case_id = 'c0000000-0000-4000-8000-000000000001' AND actor_label = 'Dalynkaa';

-- Срез чата: общий и локальный чат, личка участников и команды нарушителя.
-- Команд аутентификации здесь нет — агент их не пишет вовсе.
INSERT INTO case_messages (case_id, at, sender_id, sender_name, channel, content)
SELECT 'c0000000-0000-4000-8000-000000000001', NOW() - v.ago, v.sender, v.name, v.channel, v.content
  FROM (VALUES
    (INTERVAL '44 minutes', 'd0000000-0000-4000-8000-000000000002'::uuid, 'AlexBuilder',  'public',  'кто-нибудь видел steve возле спавна?'),
    (INTERVAL '43 minutes', 'd0000000-0000-4000-8000-000000000001'::uuid, 'GrieferSteve', 'public',  'я тут, а что'),
    (INTERVAL '42 minutes', 'd0000000-0000-4000-8000-000000000002'::uuid, 'AlexBuilder',  'public',  'не подходи к моему дому пожалуйста'),
    (INTERVAL '42 minutes', 'd0000000-0000-4000-8000-000000000001'::uuid, 'GrieferSteve', 'public',  'это не твоя земля вообще-то'),
    (INTERVAL '41 minutes', 'd0000000-0000-4000-8000-000000000001'::uuid, 'GrieferSteve', 'command', '/home base'),
    (INTERVAL '40 minutes', 'd0000000-0000-4000-8000-000000000001'::uuid, 'GrieferSteve', 'private', 'сейчас узнаешь чья это земля'),
    (INTERVAL '39 minutes', 'd0000000-0000-4000-8000-000000000002'::uuid, 'AlexBuilder',  'local',   'ты серьёзно лаву льёшь?'),
    (INTERVAL '39 minutes', 'd0000000-0000-4000-8000-000000000001'::uuid, 'GrieferSteve', 'local',   'ой, руки дрогнули'),
    (INTERVAL '38 minutes', 'd0000000-0000-4000-8000-000000000002'::uuid, 'AlexBuilder',  'public',  'модератор есть онлайн? тут griefer лавой залил полдома'),
    (INTERVAL '37 minutes', 'd0000000-0000-4000-8000-000000000001'::uuid, 'GrieferSteve', 'public',  'докажи что это я'),
    (INTERVAL '33 minutes', 'd0000000-0000-4000-8000-000000000003'::uuid, 'MinerJoe',     'public',  'у меня склад пустой, я там алмазы держал'),
    (INTERVAL '32 minutes', 'd0000000-0000-4000-8000-000000000001'::uuid, 'GrieferSteve', 'command', '/tpa AlexBuilder'),
    (INTERVAL '30 minutes', 'd0000000-0000-4000-8000-000000000001'::uuid, 'GrieferSteve', 'public',  'да идите вы все'),
    (INTERVAL '29 minutes', 'd0000000-0000-4000-8000-000000000003'::uuid, 'MinerJoe',     'private', 'алекс, у тебя тоже пропало?')
  ) AS v(ago, sender, name, channel, content);

COMMIT;
