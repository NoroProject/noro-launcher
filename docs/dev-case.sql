-- Тестовое дело для локальной проверки страницы /admin/cases.
--
-- Это seed для разработки, а не миграция: у всех записей фиксированные UUID,
-- поэтому повторный запуск переписывает то же самое и ничего не плодит.

BEGIN;

-- Участники. Наказанный, двое жалобщиков и тот, кто ведёт разбор.
--
-- `is_local_account` — не украшение: аккаунт без Discord иначе не проходит
-- users_identity_check, а заводить тестовым игрокам чужие discord_id незачем.
INSERT INTO users (id, mc_uuid, mc_username, is_local_account, created_at) VALUES
  ('d0000000-0000-4000-8000-000000000001', 'd0000000-0000-4000-8000-0000000000a1', 'GrieferSteve', TRUE, NOW() - INTERVAL '40 days'),
  ('d0000000-0000-4000-8000-000000000002', 'd0000000-0000-4000-8000-0000000000a2', 'AlexBuilder',  TRUE, NOW() - INTERVAL '90 days'),
  ('d0000000-0000-4000-8000-000000000003', 'd0000000-0000-4000-8000-0000000000a3', 'MinerJoe',     TRUE, NOW() - INTERVAL '120 days')
ON CONFLICT (id) DO UPDATE SET mc_username = EXCLUDED.mc_username;

-- Дело в работе: его ведёт Dalynkaa, кнопки в карточке живые.
INSERT INTO cases (id, target_id, game_server_id, status, claimed_by, claimed_at, opened_at)
SELECT 'c0000000-0000-4000-8000-000000000001',
       'd0000000-0000-4000-8000-000000000001',
       gs.id, 'in_review',
       (SELECT id FROM users WHERE mc_username = 'Dalynkaa'),
       NOW() - INTERVAL '14 minutes',
       NOW() - INTERVAL '38 minutes'
  FROM game_servers gs WHERE gs.name = 'Main'
ON CONFLICT (id) DO UPDATE SET
  status = EXCLUDED.status, claimed_by = EXCLUDED.claimed_by,
  claimed_at = EXCLUDED.claimed_at, opened_at = EXCLUDED.opened_at;

-- Три жалобы от двух разных людей: в очереди это «3 жалобы от 2 человек».
INSERT INTO player_reports
  (id, reporter_id, target_id, game_server_id, reason, world, x, y, z, status, case_id, created_at)
SELECT v.id, v.reporter, 'd0000000-0000-4000-8000-000000000001', gs.id, v.reason,
       'minecraft:overworld', v.x, v.y, v.z, 'open',
       'c0000000-0000-4000-8000-000000000001', NOW() - v.ago
  FROM game_servers gs,
       (VALUES
         ('e0000000-0000-4000-8000-000000000001'::uuid, 'd0000000-0000-4000-8000-000000000002'::uuid,
          'Снёс половину моего дома и залил лавой, пока я был в шахте', 412.0, 71.0, -338.0, INTERVAL '38 minutes'),
         ('e0000000-0000-4000-8000-000000000002'::uuid, 'd0000000-0000-4000-8000-000000000003'::uuid,
          'Он же вынес мои сундуки на общем складе клана', 405.0, 68.0, -351.0, INTERVAL '31 minutes'),
         ('e0000000-0000-4000-8000-000000000003'::uuid, 'd0000000-0000-4000-8000-000000000002'::uuid,
          'Продолжает ломать, я не успеваю чинить', 418.0, 72.0, -330.0, INTERVAL '19 minutes')
       ) AS v(id, reporter, reason, x, y, z, ago)
 WHERE gs.name = 'Main'
ON CONFLICT (id) DO UPDATE SET reason = EXCLUDED.reason, case_id = EXCLUDED.case_id;

-- Наказание, выданное из разбора: мут за чат, пока идёт проверка гриферства.
INSERT INTO punishments
  (id, user_id, kind, reason, actor_id, actor_label, expires_at, rule_id, rule_code, case_id, created_at)
SELECT 'f0000000-0000-4000-8000-000000000001',
       'd0000000-0000-4000-8000-000000000001', 'mute', 'Оскорбление участников',
       u.id, 'Dalynkaa', NOW() + INTERVAL '110 minutes',
       r.id, r.code, 'c0000000-0000-4000-8000-000000000001', NOW() - INTERVAL '9 minutes'
  FROM users u, rules r
 WHERE u.mc_username = 'Dalynkaa' AND r.code = '1.1'
ON CONFLICT (id) DO UPDATE SET case_id = EXCLUDED.case_id, expires_at = EXCLUDED.expires_at;

COMMIT;
