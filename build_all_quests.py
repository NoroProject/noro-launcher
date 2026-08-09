#!/usr/bin/env python3
"""
Генератор полной квестовой книги FTB Quests на русском языке для сборки SPCreate2-1.
Создаёт 21 главу, включая вкладку «Общая прогрессия» и полную цепочку Tiny Mob Farm.
"""
import hashlib
import json
import os
import shutil
import sys

TARGET_DIR = '/Users/dalynkaa/Downloads/SPCreate2-1/world/ftbquests/quests'
INDEX_FILE = '/private/tmp/claude-501/-Users-dalynkaa-Downloads-SPCreate2-1-mods/c6efa2ac-952e-4bbb-afc3-fe72bd42b3d2/scratchpad/items_index.json'

if os.path.exists(INDEX_FILE):
    with open(INDEX_FILE, 'r', encoding='utf-8') as f:
        VALID_ITEMS = json.load(f)
else:
    VALID_ITEMS = {}

_used_ids = set()

def qid(seed):
    h = hashlib.sha1(seed.encode('utf-8')).hexdigest()[:16].upper()
    while h in _used_ids:
        seed += '!'
        h = hashlib.sha1(seed.encode('utf-8')).hexdigest()[:16].upper()
    _used_ids.add(h)
    return h

def esc(s):
    return s.replace('\\', '\\\\').replace('"', '\\"')

def validate_item(item_str):
    if ':' not in item_str:
        return
    ns, name = item_str.split(':', 1)
    if ns in ('minecraft', 'spcreate'):
        return
    if ns in VALID_ITEMS:
        if name not in VALID_ITEMS[ns]:
            print(f"[WARN] Item '{item_str}' not found in index for namespace '{ns}'")
    else:
        print(f"[WARN] Namespace '{ns}' not found in item index")

class Chapter:
    def __init__(self, fname, title, icon, group_id, subtitle=''):
        self.fname = fname
        self.title = title
        self.icon = icon
        self.group_id = group_id
        self.subtitle = subtitle
        self.quests = []
        self.by_key = {}
        validate_item(icon)

    def q(self, key, title, x, y, desc, tasks, rewards=(), deps=(), shape='', size=0.0, optional=False):
        self.by_key[key] = qid(f"{self.fname}/{key}/q")
        self.quests.append({
            'key': key,
            'title': title,
            'x': float(x),
            'y': float(y),
            'desc': desc,
            'tasks': tasks,
            'rewards': rewards,
            'deps': deps,
            'shape': shape,
            'size': float(size),
            'optional': optional
        })
        return key

    def render(self, order_index):
        lines = []
        lines.append('{')
        lines.append(f'\tid: "{qid(self.fname + "/chapter")}"')
        lines.append(f'\tgroup: "{self.group_id}"')
        lines.append(f'\torder_index: {order_index}')
        lines.append(f'\tfilename: "{self.fname}"')
        lines.append(f'\ttitle: "{esc(self.title)}"')
        if self.subtitle:
            lines.append(f'\tsubtitle: ["{esc(self.subtitle)}"]')
        lines.append(f'\ticon: "{self.icon}"')
        lines.append('\tdefault_quest_shape: ""')
        lines.append('\tdefault_hide_dependency_lines: false')
        lines.append('\tquests: [')

        for idx, qd in enumerate(self.quests):
            lines.append('\t\t{')
            lines.append(f'\t\t\tid: "{self.by_key[qd["key"]]}"')
            lines.append(f'\t\t\tx: {qd["x"]:.1f}d')
            lines.append(f'\t\t\ty: {qd["y"]:.1f}d')
            lines.append(f'\t\t\ttitle: "{esc(qd["title"])}"')
            if qd['shape']:
                lines.append(f'\t\t\tshape: "{qd["shape"]}"')
            if qd['size'] > 0:
                lines.append(f'\t\t\tsize: {qd["size"]:.1f}d')
            if qd['optional']:
                lines.append('\t\t\toptional: true')
            if qd['desc']:
                body = ', '.join(f'"{esc(line)}"' for line in qd['desc'])
                lines.append(f'\t\t\tdescription: [{body}]')
            if qd['deps']:
                dep_ids = []
                for d in qd['deps']:
                    if d in self.by_key:
                        dep_ids.append(f'"{self.by_key[d]}"')
                    else:
                        print(f"[ERROR] Dep '{d}' not found in chapter '{self.fname}'")
                lines.append(f'\t\t\tdependencies: [{", ".join(dep_ids)}]')

            # Render Tasks
            task_str_list = []
            for t_idx, task_spec in enumerate(qd['tasks']):
                task_seed = f"{self.fname}/{qd['key']}/t{t_idx}"
                if isinstance(task_spec, dict):
                    ttype = task_spec['type']
                    if ttype == 'checkmark':
                        task_str_list.append(f'{{ id: "{qid(task_seed)}", type: "checkmark", title: "{esc(task_spec["title"])}" }}')
                    elif ttype == 'kill':
                        task_str_list.append(f'{{ id: "{qid(task_seed)}", type: "kill", entity: "{task_spec["entity"]}", value: {task_spec.get("count", 1)}L }}')
                    elif ttype == 'advancement':
                        task_str_list.append(f'{{ id: "{qid(task_seed)}", type: "advancement", advancement: "{task_spec["advancement"]}", criterion: "" }}')
                else:
                    if isinstance(task_spec, str):
                        item, count = task_spec, 1
                    else:
                        item, count = task_spec[0], task_spec[1]
                    validate_item(item)
                    t_str = f'{{ id: "{qid(task_seed)}", type: "item", item: "{item}"'
                    if count > 1:
                        t_str += f', count: {count}L'
                    t_str += ' }'
                    task_str_list.append(t_str)
            lines.append(f'\t\t\ttasks: [{", ".join(task_str_list)}]')

            # Render Rewards
            reward_str_list = []
            for r_idx, r_spec in enumerate(qd['rewards']):
                r_seed = f"{self.fname}/{qd['key']}/r{r_idx}"
                rtype = r_spec[0]
                if rtype == 'item':
                    item = r_spec[1]
                    count = r_spec[2] if len(r_spec) > 2 else 1
                    validate_item(item)
                    r_str = f'{{ id: "{qid(r_seed)}", type: "item", item: "{item}"'
                    if count > 1:
                        r_str += f', count: {count}'
                    r_str += ' }'
                    reward_str_list.append(r_str)
                elif rtype == 'xp':
                    reward_str_list.append(f'{{ id: "{qid(r_seed)}", type: "xp_levels", xp_levels: {r_spec[1]} }}')
            if reward_str_list:
                lines.append(f'\t\t\trewards: [{", ".join(reward_str_list)}]')

            q_end = '\t\t},' if idx < len(self.quests) - 1 else '\t\t}'
            lines.append(q_end)
        lines.append('\t]')
        lines.append('\tquest_links: [ ]')
        lines.append('}')
        return '\n'.join(lines) + '\n'

GROUPS = []
CHAPTERS = []

def add_group(title):
    gid = qid('group/' + title)
    GROUPS.append((gid, title))
    return gid

def add_chapter(fname, title, icon, group_id, subtitle=''):
    ch = Chapter(fname, title, icon, group_id, subtitle)
    CHAPTERS.append(ch)
    return ch

# ==================== СОЗДАНИЕ ГРУПП И ГЛАВ ====================

g_start = add_group("Начало")
g_create = add_group("Create")
g_industry = add_group("Индустрия")
g_war = add_group("Военное дело")
g_food = add_group("Кулинария и Фермерство")
g_misc = add_group("Торговля и Медиа")

# --- 0. ОБЩАЯ ПРОГРЕССИЯ СБОРКИ (NEW) ---
c = add_chapter('progression', 'Общая прогрессия сборки', 'create_tools_n_weapons:gear_pickaxe', g_start, 'Путеводитель по всем этапам развития')

c.q('p1_wood_stone', '1. Дерево и Камень', 0, 0, [
    'Старт игры: сруби дерево, сделай верстак и каменную кирку.',
    '',
    'Важно: ванильные инструменты из железа, золота и алмазов вырезаны!',
    'Развитие снаряжения идёт по цепочке Медь -> Бронза -> Сталь -> Незерит.'
], ['minecraft:stone_pickaxe'], [('item', 'minecraft:torch', 16)], shape='gear', size=1.5)

c.q('p2_copper', '2. Медный век', 2, 0, [
    'Первые металлы: Медь и Цинк.',
    '',
    'Сруби медную руду и переплавь слитки.',
    'Медный инструмент копает быстрее камня и позволяет добывать цинк.'
], ['create_ironworks:copper_pickaxe'], [('item', 'minecraft:copper_ingot', 8)], deps=['p1_wood_stone'])

c.q('p3a_bronze_ingot', '3. Сплав Бронзы', 4, 0, [
    'Первый настоящий сплав сборки: Бронза!',
    '',
    'Как получить бронзу:',
    'Сплавь 3 Медных слитка + 1 Цинковый слиток на Верстаке или в Чане Create.',
    'Бронзовые слитки нужны для прочной брони, инструментов и пушек.'
], [('create_ironworks:bronze_ingot', 4)], [('item', 'create_ironworks:bronze_ingot', 4)], deps=['p2_copper'], shape='diamond', size=1.3)

c.q('p3b_bronze_tools', '3.1 Бронзовый инструмент', 6, 0, [
    'Из полученных бронзовых слитков скрафти Бронзовую кирку.',
    '',
    'Бронзовые инструменты прочнее ванильного железа.',
    'Из бронзы также делается Паксель (кирка+топор+лопата в одном).'
], ['create_ironworks:bronze_pickaxe'], [('xp', 10)], deps=['p3a_bronze_ingot'])

c.q('p4_andesite', '4. Андезитовый сплав и Ключ', 8, 0, [
    'Старт механики Create:',
    '- Смешай андезит с цинком или железом для Андезитового сплава.',
    '- Сделай Гаечный ключ, валы и андезитовые корпуса.'
], ['create:andesite_casing', 'create:wrench'], [('item', 'create:andesite_alloy', 16)], deps=['p3b_bronze_tools'])

c.q('p5_first_machines', '5. Первые машины: Жёрнов и Пресс', 10, 0, [
    'Первая автоматизация:',
    '- Жёрнов дробит руды и зерно.',
    '- Механический пресс штампует железные и медные листы.'
], ['create:millstone', 'create:mechanical_press'], [('item', 'create:iron_sheet', 4)], deps=['p4_andesite'])

c.q('p6_netherless_blaze', '6. Netherless: Пустая горелка', 12, 0, [
    'В этой сборке путь в Незер не обязателен!',
    '',
    'Скрафти Пустую горелку всполоха (Empty Blaze Burner).',
    'Затем поймай всполоха или создай огненный порошок через Netherless-крафты.'
], ['create:empty_blaze_burner'], [('item', 'create:empty_blaze_burner', 1)], deps=['p5_first_machines'], shape='diamond')

c.q('p7_heated_brass', '7. Горелка и Нагрев Латуни', 14, 0, [
    'Получение Латуни требует НАГРЕВА!',
    '',
    'Как получить Латунь:',
    '1. Поставь Горелку Всполоха (Blaze Burner) под Чан с Миксером.',
    '2. Зажги горелку огненным порошком или углём.',
    '3. Смешай Медный и Цинковый слиток в нагретом чане для выходы Латуни!'
], ['create:blaze_burner', ('create:brass_ingot', 4)], [('item', 'create:brass_ingot', 8)], deps=['p6_netherless_blaze'], shape='diamond', size=1.4)

c.q('p8_brass_logistics', '8. Латунные корпуса и Умная логистика', 16, 0, [
    'Латунь открывает точную логистику:',
    '- Латунные корпуса и хранилища.',
    '- Умные воронки, туннели-сортировщики и фильтры.'
], ['create:brass_casing', 'create:smart_chute'], [('item', 'create:brass_ingot', 4)], deps=['p7_heated_brass'])

c.q('p9_cast_iron', '9. Доменная печь и Чугун TFMG', 18, 0, [
    'Старт тяжелопромышленного мода TFMG:',
    'Выплавляй Чугун из железной руды и угля в доменной печи.',
    'Чугун нужен для крупных машин и реакторов.'
], ['tfmg:cast_iron_ingot'], [('item', 'tfmg:cast_iron_ingot', 8)], deps=['p8_brass_logistics'])

c.q('p10_vat', '10. Химический чан TFMG', 20, 0, [
    'Собери многоблочный Химический Чан TFMG.',
    'Он открывает химические реакции, растворы, серу и выщелачивание.'
], ['tfmg:cast_iron_chemical_vat'], [('item', 'tfmg:cast_iron_ingot', 8)], deps=['p9_cast_iron'])

c.q('p11_steel', '11. Стальной век', 22, 0, [
    'Выплави Сталь из чугуна/угля в печи TFMG.',
    '',
    'Стальная броня и стальной инструмент — лучшая экипировка перед Незеритом.',
    'Прессованные стальные листы нужны для вышек и ракет.'
], ['create_ironworks:steel_pickaxe', 'create_ironworks:steel_ingot'], [('item', 'create_ironworks:steel_ingot', 8)], deps=['p10_vat'], shape='diamond')

c.q('p12_oil_pumpjack', '12. Вышка добычи нефти', 24, 0, [
    'Построй нефтяную вышку (Pumpjack) TFMG на залежи нефти.',
    'Добывай Сырую нефть в цистерны.'
], ['tfmg:large_pumpjack_hammer_head', 'tfmg:crude_oil_bucket'], [('item', 'tfmg:crude_oil_bucket', 1)], deps=['p11_steel'])

c.q('p13_distillation_plastic', '13. Перегонка нефти и Пластик', 26, 0, [
    'Построй Дистилляционную колонну TFMG.',
    'Раздели нефть на 5 фракций (газ, бензин, керосин, дизель, мазут).',
    'Синтезируй Пластик из нафты.'
], [('tfmg:diesel_bucket', 1), 'tfmg:plastic_sheet'], [('item', 'tfmg:plastic_block', 4)], deps=['p12_oil_pumpjack'])

c.q('p14_electronics', '14. Электроника и Схемы', 28, 0, [
    'Трави печатные платы кислотой в чане.',
    'Собери Стальной механизм (Steel Mechanism) — основу сложной техники.'
], ['tfmg:circuit_board', 'tfmg:steel_mechanism'], [('item', 'tfmg:steel_mechanism', 2)], deps=['p13_distillation_plastic'])

c.q('p15_precision_red', '15. Точный механизм и Красный цинк', 30, 0, [
    'Настрой последовательную сборочную линию из депо и установщиков.',
    'Собери Точный механизм и Красный цинк.'
], ['create:precision_mechanism', 'spcreate:zinc_red'], [('item', 'create:precision_mechanism', 1)], deps=['p14_electronics'])

c.q('p16_electrum', '16. Сплав Электрума', 32, 0, [
    'Сплавь Золото и Серебро в чане TFMG для получения Электрума.',
    '',
    'Помни: именно Электрум заменил золото в рецепте Незерита!'
], ['spcreate:electrum_ingot'], [('item', 'spcreate:electrum_ingot', 4)], deps=['p15_precision_red'], shape='diamond')

c.q('p17_netherite', '17. Незеритовый век', 34, 0, [
    'Рецепт Незерита изменён:',
    '4 Незеритовых обломка + 4 слитка Электрума в чане.'
], ['minecraft:netherite_ingot'], [('xp', 30)], deps=['p16_electrum'])

c.q('p18_rainbow_diamond', '18. Усиленный Алмаз', 36, 0, [
    'Обработай Алмаз пятью видами топлива TFMG на сборочной линии.',
    'Получи Радужный / Усиленный Алмаз (Rainbow Diamond).'
], ['spcreate:rainbow_diamond'], [('xp', 40)], deps=['p17_netherite'])

c.q('p19_endgame', '19. Финал: Гир-инструменты и Незеритовый Джетпак', 38, 0, [
    'Вершина развития:',
    '- Гир-инструменты (Create Tools & Weapons) на незерите и усиленном алмазе.',
    '- Незеритовый джетпак с бесконечным полётом.'
], ['create_tools_n_weapons:gear_pickaxe', 'create_jetpack:netherite_jetpack'], [('xp', 50)], deps=['p18_rainbow_diamond'], shape='hexagon', size=1.5)


# --- 1. НАЧАЛО: ПЕРВЫЕ ШАГИ ---
c = add_chapter('start', 'Первые шаги', 'minecraft:oak_log', g_start, 'Добро пожаловать в SPCreate 2.1!')
c.q('wood', 'Дерево', 0, 0, [
    'Всё начинается здесь. Сруби дерево и сделай верстак.',
    '',
    'В этой сборке привычная цепочка «железо → золото → алмаз» отключена.',
    'Инструменты идут другим путём — через медь и бронзу.'
], [('minecraft:oak_log', 8)], [('item', 'minecraft:crafting_table', 1)], shape='gear', size=1.5)

c.q('stone', 'Каменный век', 2, 0, [
    'Каменные инструменты работают как обычно — с них и начинаем.'
], ['minecraft:stone_pickaxe'], [('item', 'minecraft:torch', 16)], deps=['wood'])

c.q('copper_ore', 'Медная жила', 4, 0, [
    'Медь — основа всего. Она нужна и для инструментов, и для механизмов Create.'
], [('minecraft:raw_copper', 16)], [('item', 'minecraft:copper_ingot', 8)], deps=['stone'])

c.q('zinc', 'Цинк', 4, 2, [
    'Цинк встречается в тех же слоях, что и медь.',
    'Вместе они дают латунь — металл точных механизмов.'
], [('create:raw_zinc', 8)], [('item', 'create:zinc_ingot', 4)], deps=['copper_ore'])

c.q('no_iron', 'Куда делось железо?', 2, 2, [
    'Железные, золотые и алмазные инструменты и броня в этой сборке не крафтятся.',
    '',
    'Само железо никуда не делось — оно нужно для механизмов.',
    'А вот снаряжение теперь куётся из меди, бронзы и стали.',
    '',
    'Загляни в главу «Кузница», чтобы начать.'
], [('minecraft:iron_ingot', 8)], [('item', 'minecraft:iron_ingot', 4)], deps=['stone'])

c.q('food', 'Первый обед', 0, 2, [
    'Голод здесь строже обычного: следи за разнообразием еды.'
], ['minecraft:bread'], [('item', 'minecraft:bread', 4)], deps=['wood'])

c.q('andesite', 'Андезитовый сплав', 6, 0, [
    'Андезитовый сплав — фундамент всей механики Create.',
    'Смешай андезит с цинком или железом.'
], [('create:andesite_alloy', 8)], [('item', 'create:andesite_alloy', 8), ('xp', 5)], deps=['zinc'], shape='diamond', size=1.5)

c.q('smithing', 'Стол кузнеца', 6, 2, [
    'Рецепт изменён: вместо досок и железа нужен андезитовый корпус.',
    'Так же переделаны наблюдатель, раздатчик, дроппер и камнерез.'
], ['minecraft:smithing_table'], [('item', 'create:andesite_alloy', 4)], deps=['andesite'], optional=True)


# --- 2. КУЗНИЦА: МЕДЬ И БРОНЗА ---
c = add_chapter('ironworks_early', 'Кузница: медь и бронза', 'create_ironworks:bronze_pickaxe', g_start, 'Замена вырезанной ванильной прогрессии')
c.q('copper_tools', 'Медный инструмент', 0, 0, [
    'Медные инструменты — первая ступень. Слабее железных, зато доступны сразу.'
], ['create_ironworks:copper_pickaxe'], [('item', 'minecraft:copper_ingot', 6)], shape='gear', size=1.5)

c.q('copper_armor', 'Медная броня', 0, 2, [
    'Полный комплект защитит на первых ночах.'
], ['create_ironworks:copper_armor_chestplate'], [('item', 'minecraft:copper_ingot', 8)], deps=['copper_tools'])

c.q('copper_hammer', 'Медный молот', 2, -2, [
    'Молот копает область 3x3. Рецепт переведён на механический крафт.'
], ['create_ironworks:copper_hammer'], [('xp', 3)], deps=['copper_tools'], optional=True)

c.q('bronze', 'Бронза', 2, 0, [
    'Бронза — сплав меди и цинка. Прочнее железа и открывает середину игры.'
], [('create_ironworks:bronze_ingot', 4)], [('item', 'create_ironworks:bronze_ingot', 4)], deps=['copper_tools'], shape='diamond')

c.q('bronze_tools', 'Бронзовый инструмент', 4, 0, [
    'Основной рабочий набор на долгое время.'
], ['create_ironworks:bronze_pickaxe'], [('item', 'create_ironworks:bronze_ingot', 6)], deps=['bronze'])

c.q('bronze_armor', 'Бронзовая броня', 4, 2, [
    'Держит удар лучше железной.'
], ['create_ironworks:bronze_armor_chestplate'], [('item', 'create_ironworks:bronze_ingot', 8)], deps=['bronze'])

c.q('paxel', 'Паксель', 4, -2, [
    'Кирка, топор и лопата в одном предмете. Собирается в механическом крафтере.'
], ['create_ironworks:bronze_paxel'], [('xp', 10)], deps=['bronze_tools'], optional=True)

c.q('brass_tools', 'Латунный инструмент', 6, 0, [
    'Латунь быстрее бронзы, но менее прочная. Хороша для точной работы.'
], ['create_ironworks:brass_pickaxe'], [('item', 'create:brass_ingot', 4)], deps=['bronze_tools'])


# --- 3. CREATE: ОСНОВЫ ---
c = add_chapter('create_basics', 'Механика Create: основы', 'create:cogwheel', g_create, 'Вращение, передача, первые машины')
c.q('wrench', 'Гаечный ключ', 0, 0, [
    'Главный инструмент инженера: поворачивает и разбирает механизмы.'
], ['create:wrench'], [('item', 'create:andesite_alloy', 4)], shape='gear', size=1.5)

c.q('goggles', 'Инженерные очки', 0, 2, [
    'Показывают скорость, напряжение и содержимое механизмов.',
    'Рецепт изменён: нужен золотой лист вместо самородков.'
], ['create:goggles'], [('xp', 5)], deps=['wrench'])

c.q('shaft', 'Валы и шестерни', 2, 0, [
    'Вал передаёт вращение по прямой, шестерня — меняет направление.'
], [('create:shaft', 8), ('create:cogwheel', 4)], [('item', 'create:andesite_alloy', 8)], deps=['wrench'])

c.q('water_wheel', 'Водяное колесо', 4, -2, [
    'Первый источник вращения. Медленный, зато бесплатный и вечный.'
], ['create:water_wheel'], [('item', 'create:shaft', 8)], deps=['shaft'])

c.q('windmill', 'Ветряк', 4, 2, [
    'Подшипник плюс паруса. Чем больше парусов, тем выше мощность.'
], ['create:windmill_bearing', ('create:sail_frame', 8)], [('item', 'create:andesite_alloy', 8)], deps=['shaft'])

c.q('gearbox', 'Коробка передач', 4, 0, [
    'Передаёт вращение за угол и между уровнями.'
], ['create:gearbox'], [('item', 'create:cogwheel', 4)], deps=['shaft'])

c.q('casing', 'Андезитовый корпус', 6, 0, [
    'Корпус превращает голые валы в аккуратные механизмы.',
    'Он же теперь нужен для верстака кузнеца, камнереза и раздатчика.'
], [('create:andesite_casing', 4)], [('item', 'create:andesite_casing', 4)], deps=['water_wheel', 'windmill', 'gearbox'], shape='diamond', size=1.5)

c.q('speed', 'Контроль скорости', 8, 2, [
    'Регулятор скорости, сцепление и передача позволяют управлять машиной.'
], ['create:gearshift', 'create:clutch'], [('xp', 5)], deps=['casing'], optional=True)


# --- 4. CREATE: ОБРАБОТКА ---
c = add_chapter('create_processing', 'Create: обработка', 'create:mechanical_press', g_create, 'Дробление, прессование, смешивание')
c.q('millstone', 'Жёрнов', 0, 0, [
    'Самый дешёвый способ дробить руду и зерно. Работает от вала снизу.'
], ['create:millstone'], [('item', 'create:andesite_alloy', 4)], shape='gear', size=1.5)

c.q('crushing', 'Дробильные колёса', 2, -2, [
    'Пара колёс дробит всё, что падает между ними. Удваивает выход руды.'
], [('create:crushing_wheel', 2)], [('item', 'minecraft:raw_iron', 8)], deps=['millstone'])

c.q('press', 'Механический пресс', 2, 0, [
    'Штампует листы из слитков. Листы нужны почти везде.'
], ['create:mechanical_press'], [('item', 'create:iron_sheet', 4)], deps=['millstone'])

c.q('basin', 'Чан и миксер', 2, 2, [
    'Смешивание — основа сплавов и жидкой химии.'
], ['create:basin', 'create:mechanical_mixer'], [('item', 'create:andesite_alloy', 8)], deps=['millstone'])

c.q('burner', 'Горелка', 4, 2, [
    'Нагревает чан. Без огня многие рецепты не пойдут.',
    'Раздуй её огненным порошком до «нагретой».'
], ['create:blaze_burner'], [('item', 'minecraft:blaze_powder', 4)], deps=['basin'])

c.q('meat', 'Мясо без животных', 6, 2, [
    'В этой сборке мясо можно смешивать в чане из овощей и жидкостей.',
    '',
    'Говядина, свинина, баранина, крольчатина, курица и яйца —',
    'всё делается на кухне, а не в загоне.'
], [('minecraft:beef', 8)], [('item', 'minecraft:beef', 8)], deps=['burner'], shape='hexagon')

c.q('saw', 'Механическая пила', 4, -2, [
    'Пилит брёвна выгоднее верстака и работает на конвейере.'
], ['create:mechanical_saw'], [('item', 'minecraft:oak_planks', 32)], deps=['crushing'])

c.q('fan', 'Вентилятор', 4, 0, [
    'Обдув через огонь плавит, через воду — моет, через лёд — морозит.'
], ['create:encased_fan'], [('xp', 5)], deps=['press'])

c.q('copper_casing', 'Медный корпус и жидкости', 6, 0, [
    'Трубы, насосы и баки — начало жидкостной логистики.'
], [('create:copper_casing', 4), ('create:fluid_pipe', 8), 'create:mechanical_pump'], [('item', 'create:copper_casing', 4)], deps=['fan'], shape='diamond')


# --- 5. CREATE: ЛОГИСТИКА И ПОЕЗДА ---
c = add_chapter('create_logistics', 'Create: логистика и поезда', 'create:track', g_create, 'Конвейеры, воронки, железные дороги')
c.q('belt', 'Механический пояс', 0, 0, [
    'Лента между двумя валами. Основа любой автоматизации.'
], [('create:belt_connector', 1)], [('item', 'create:andesite_alloy', 8)], shape='gear', size=1.5)

c.q('funnel', 'Воронки', 2, 0, [
    'Загружают и разгружают ленты, сундуки и машины.'
], [('create:andesite_funnel', 4), ('create:andesite_tunnel', 2)], [('item', 'create:andesite_alloy', 8)], deps=['belt'])

c.q('brass', 'Латунный корпус', 4, 0, [
    'Латунь открывает умную логистику: фильтры, умные воронки, туннели-сортировщики.'
], [('create:brass_casing', 4)], [('item', 'create:brass_ingot', 8)], deps=['funnel'], shape='diamond')

c.q('smart', 'Умная сортировка', 6, -2, [
    'Умные воронки и латунные туннели умеют фильтровать поток.'
], [('create:smart_chute', 1), ('create:brass_tunnel', 2)], [('xp', 10)], deps=['brass'])

c.q('vault', 'Хранилище', 6, 2, [
    'Хранилища объединяются в один большой склад.'
], [('create:item_vault', 4)], [('item', 'create:brass_ingot', 4)], deps=['brass'])

c.q('train_casing', 'Железнодорожный корпус', 8, 0, [
    'Он же нужен для кафедры и фотостойки Exposure.'
], [('create:railway_casing', 2)], [('item', 'create:railway_casing', 2)], deps=['brass'])

c.q('track', 'Рельсы', 10, 0, [
    'Поезда Create ездят по своим путям и умеют работать по расписанию.'
], [('create:track', 16)], [('item', 'create:track', 32)], deps=['train_casing'])

c.q('station', 'Станция и состав', 12, 0, [
    'Станция, тележки и расписание — и поезд поедет сам.'
], ['create:track_station', ('create:railway_casing', 2), 'create:schedule'], [('xp', 20)], deps=['track'], shape='hexagon', size=1.5)

c.q('trainmap', 'Карта поездов', 12, 2, [
    'Мод Xaero Train Map показывает все поезда и станции прямо на карте мира.'
], [{'type': 'checkmark', 'title': 'Открыть карту мира (клавиша M)'}], [('xp', 5)], deps=['station'], optional=True)


# --- 6. CREATE: ВЫСОКИЕ ТЕХНОЛОГИИ ---
c = add_chapter('create_advanced', 'Create: высокие технологии', 'create:precision_mechanism', g_create, 'Точные механизмы и сборочные линии')
c.q('crafter', 'Механический крафтер', 0, 0, [
    'Собирает предметы по узору. В этой сборке через него идут почти все',
    'важные рецепты: инструменты, снаряды, джетпаки.'
], [('create:mechanical_crafter', 5)], [('item', 'create:brass_ingot', 8)], shape='gear', size=1.5)

c.q('deployer', 'Установщик', 2, -2, [
    'Повторяет действие руки: кликает, пашет, атакует, крафтит.'
], [('create:deployer', 2)], [('item', 'create:andesite_alloy', 8)], deps=['crafter'])

c.q('sequenced', 'Сборочная линия', 2, 0, [
    'Последовательная сборка: предмет проходит несколько станций подряд.',
    'По этому принципу делаются красный цинк и усиленный алмаз.'
], ['create:depot', ('create:belt_connector', 1)], [('xp', 10)], deps=['crafter'])

c.q('electron', 'Электронная лампа', 4, 0, [
    'Лампа нужна почти всем продвинутым машинам.'
], [('create:electron_tube', 4)], [('item', 'create:electron_tube', 2)], deps=['sequenced'])

c.q('precision', 'Точный механизм', 6, 0, [
    'Вершина сборочных линий Create.'
], [('create:precision_mechanism', 2)], [('item', 'create:precision_mechanism', 1), ('xp', 20)], deps=['electron'], shape='diamond', size=1.5)

c.q('zinc_red', 'Красный цинк', 6, -2, [
    'Собственный предмет сборки. Делается на сборочной линии из цинка,',
    'смолы, нафты и точного механизма.'
], ['spcreate:zinc_red'], [('xp', 15)], deps=['precision'])

c.q('rainbow', 'Усиленный алмаз', 8, 0, [
    'Алмаз, пропитанный пятью видами топлива TFMG.',
    '',
    'Нужен для гир-инструментов и топовых снарядов.'
], ['spcreate:rainbow_diamond'], [('xp', 30)], deps=['precision'], shape='hexagon', size=1.5)

c.q('arm', 'Механическая рука', 4, 2, [
    'Перекладывает предметы между несколькими точками по фильтру.'
], ['create:mechanical_arm'], [('xp', 10)], deps=['electron'], optional=True)


# --- 7. СТАЛЬ И ЭЛЕКТРУМ ---
c = add_chapter('ironworks_late', 'Сталь и электрум', 'create_ironworks:steel_ingot', g_create, 'Поздняя металлургия сборки')
c.q('steel', 'Сталь', 0, 0, [
    'В сборке сталь только одна — из Create Ironworks.',
    'Все дубликаты стали из других модов сведены к ней.'
], [('create_ironworks:steel_ingot', 4)], [('item', 'create_ironworks:steel_ingot', 4)], shape='gear', size=1.5)

c.q('steel_tools', 'Стальной инструмент', 2, -2, [
    'Лучший обычный инструмент до незерита.'
], ['create_ironworks:steel_pickaxe'], [('item', 'create_ironworks:steel_ingot', 4)], deps=['steel'])

c.q('steel_armor', 'Стальная броня', 2, 2, [
    'Надёжная защита для середины игры.'
], ['create_ironworks:steel_armor_chestplate'], [('item', 'create_ironworks:steel_ingot', 6)], deps=['steel'])

c.q('steel_sheet', 'Стальной лист', 2, 0, [
    'Прессуется из слитка. Нужен ракетам и корпусам.'
], [('create_ironworks:steel_sheet', 8)], [('item', 'create_ironworks:steel_sheet', 4)], deps=['steel'])

c.q('electrum', 'Электрум', 4, 0, [
    'Сплав золота и серебра — собственный металл сборки.',
    '',
    'Внимание: именно электрум теперь нужен для незеритового слитка.'
], [('spcreate:electrum_ingot', 4)], [('item', 'spcreate:electrum_ingot', 2)], deps=['steel_sheet'], shape='diamond')

c.q('netherite', 'Незерит', 6, 0, [
    'Рецепт изменён: 4 обломка незерита + 4 слитка электрума.',
    'Золотые слитки больше не участвуют.'
], ['minecraft:netherite_ingot'], [('xp', 30)], deps=['electrum'], shape='hexagon', size=1.5)

c.q('gear_tools', 'Гир-инструменты', 8, 0, [
    'Механические инструменты Create Tools and Weapons.',
    'Собираются в крафтере из латунных листов, стального механизма,',
    'усиленного алмаза и незеритового инструмента.'
], ['create_tools_n_weapons:gear_pickaxe'], [('xp', 50)], deps=['netherite'], size=1.5)


# --- 8. ДОПОЛНЕНИЯ CREATE ---
c = add_chapter('create_addons', 'Дополнения Create', 'create_jetpack:jetpack', g_create, 'Джетпаки, печати, нарезка и мобы')
c.q('jetpack', 'Медный джетпак', 0, 0, [
    'Работает на воздухе из медного бака. Позволяет летать!',
    'Собирается в механическом крафтере.'
], ['create_jetpack:jetpack'], [('xp', 15)], shape='gear', size=1.5)

c.q('netherite_jetpack', 'Незеритовый джетпак', 2, 0, [
    'Продвинутый джетпак: летит быстрее и держит больше воздуха.'
], ['create_jetpack:netherite_jetpack'], [('xp', 35)], deps=['jetpack'], shape='hexagon')

c.q('enchanting', 'Печать книг и чар', 0, 2, [
    'Create Enchantment Industry позволяет печатать чары на печатном прессе',
    'и собирать жидкий опыт из мобов.'
], ['create_enchantment_industry:printer'], [('xp', 10)])

c.q('blaze_enchanter', 'Печь чар', 2, 2, [
    'Печатный пресс с печью всполоха автоматизирует наложение чар.'
], ['create_enchantment_industry:blaze_enchanter'], [('xp', 20)], deps=['enchanting'])

c.q('slicer', 'Нарезчик овощей', 4, 0, [
    'Create Slice & Dice добавляет авто-нарезку на конвейере.'
], ['sliceanddice:slicer'], [('item', 'farmersdelight:cabbage', 8)], optional=True)

c.q('copycats', 'Блоки-мимики', 4, 2, [
    'Create Copycats и Aero Copycats принимают текстуру любого блока.'
], ['copycats:copycat_block'], [('xp', 5)], deps=['slicer'], optional=True)

c.q('dragons', 'Драконы и реликвии', 6, 0, [
    'Create Dragons Plus добавляет механических драконов и особые реликвии.'
], [{'type': 'checkmark', 'title': 'Исследовать моды драконов'}], [('xp', 20)], deps=['copycats'], optional=True)


# --- 9. TFMG: НЕФТЬ И ХИМИЯ ---
c = add_chapter('tfmg_oil', 'TFMG: нефть и химия', 'tfmg:crude_oil_bucket', g_industry, 'Тяжёлая промышленность начинается здесь')
c.q('cast_iron', 'Чугун', 0, 0, [
    'Первый металл TFMG. Дешевле стали и нужен в огромных количествах.'
], [('tfmg:cast_iron_ingot', 8)], [('item', 'tfmg:cast_iron_ingot', 4)], shape='gear', size=1.5)

c.q('vat', 'Химический чан', 2, 0, [
    'Многоблочный реактор. Основа всей химии TFMG.'
], [('tfmg:cast_iron_chemical_vat', 4)], [('item', 'tfmg:cast_iron_ingot', 8)], deps=['cast_iron'])

c.q('pumpjack', 'Нефтяной насос', 4, 0, [
    'Качает сырую нефть из подземных залежей.'
], ['tfmg:large_pumpjack_hammer_head'], [('xp', 15)], deps=['vat'])

c.q('oil', 'Сырая нефть', 6, 0, [
    'Чёрное золото. Без неё не будет ни топлива, ни пластика.'
], [('tfmg:crude_oil_bucket', 1)], [('item', 'tfmg:crude_oil_bucket', 1)], deps=['pumpjack'], shape='diamond')

c.q('distill', 'Перегонка', 8, 0, [
    'Дистилляционная колонна разделяет нефть на фракции:',
    'сжиженный газ, бензин, керосин, дизель и мазут.',
    '',
    'Все пять нужны для усиленного алмаза.'
], [('tfmg:diesel_bucket', 1)], [('xp', 25)], deps=['oil'], shape='hexagon', size=1.5)

c.q('plastic', 'Пластик', 10, 0, [
    'Продукт нефтехимии. Идёт на трубы и корпуса.'
], [('tfmg:plastic_sheet', 8)], [('item', 'tfmg:plastic_block', 4)], deps=['distill'])

c.q('sulfur', 'Сера', 4, 2, [
    'Сера добывается из серного блока и нужна для кислоты.',
    'В сборке серу также можно получить из гладкого базальта.'
], [('tfmg:sulfur_dust', 4)], [('item', 'tfmg:sulfur_dust', 4)], deps=['vat'], optional=True)

c.q('aluminum', 'Алюминий', 6, 2, [
    'Лёгкий металл из бокситов. Электролиз идёт в чане.'
], [('tfmg:aluminum_ingot', 8)], [('item', 'tfmg:aluminum_ingot', 4)], deps=['vat'])


# --- 10. TFMG: ЭЛЕКТРИКА ---
c = add_chapter('tfmg_electric', 'TFMG: электрика', 'tfmg:circuit_board', g_industry, 'Платы, двигатели, механизмы')
c.q('wire', 'Проволока', 0, 0, [
    'Медная и алюминиевая проволока — начало электрики.'
], [('tfmg:copper_wire', 8)], [('item', 'tfmg:copper_wire', 8)], shape='gear', size=1.5)

c.q('board', 'Печатная плата', 2, 0, [
    'Травится кислотой в чане. Нужна почти всем машинам сборки.'
], [('tfmg:circuit_board', 4)], [('item', 'tfmg:circuit_board', 2)], deps=['wire'], shape='diamond')

c.q('mechanism', 'Стальной механизм', 4, 0, [
    'Ключевой компонент. В этой сборке он нужен джетпакам, гир-инструментам,',
    'снарядам, телевизорам и почти всему дорогому.'
], [('tfmg:steel_mechanism', 4)], [('item', 'tfmg:steel_mechanism', 2), ('xp', 20)], deps=['board'], shape='hexagon', size=1.5)

c.q('engine', 'Двигатель', 6, -2, [
    'Дизельный двигатель даёт огромную мощность вращения.'
], ['tfmg:engine_cylinder', 'tfmg:engine_controller'], [('xp', 20)], deps=['mechanism'])

c.q('spark', 'Свеча зажигания', 6, 2, [
    'Нужна двигателям и литиевому клинку.'
], [('tfmg:spark_plug', 2)], [('item', 'tfmg:spark_plug', 2)], deps=['mechanism'])

c.q('blade', 'Литиевый клинок', 8, 2, [
    'Рецепт изменён: собирается из стального меча Ironworks,',
    'алюминиевых листов, свечи, платы и медной проволоки.'
], ['tfmg:lithium_blade'], [('xp', 30)], deps=['spark'], optional=True)


# --- 11. ЭНЕРГЕТИКА ---
c = add_chapter('power', 'Энергетика', 'powergrid:battery', g_industry, 'Электричество и красный камень')
c.q('battery', 'Батарея', 0, 0, [
    'PowerGrid приносит настоящее электричество с напряжением и током.'
], ['powergrid:battery'], [('xp', 10)], shape='gear', size=1.5)

c.q('pg_board', 'Схемотехника', 2, 0, [
    'Стол проектирования позволяет собирать собственные микросхемы.'
], ['powergrid:circuit_design_table'], [('xp', 15)], deps=['battery'])

c.q('motor', 'Электромотор', 4, 0, [
    'Превращает ток обратно во вращение Create.'
], ['powergrid:constant_speed_motor'], [('xp', 20)], deps=['pg_board'])

c.q('morered', 'Красный камень нового поколения', 2, 2, [
    'More Red добавляет цветные провода, логику и компактные схемы.'
], [{'type': 'checkmark', 'title': 'Изучить блоки More Red в JEI'}], [('item', 'minecraft:redstone', 16)], deps=['battery'], optional=True)

c.q('powerchip', 'Микросхемы', 4, 2, [
    'PowerChip позволяет упаковать сложную логику в один блок.'
], [{'type': 'checkmark', 'title': 'Собрать первую микросхему'}], [('xp', 10)], deps=['morered'], optional=True)


# --- 12. АВТОМАТИЗАЦИЯ ---
c = add_chapter('automation', 'Автоматизация', 'computercraft:computer_normal', g_industry, 'Компьютеры и программирование')
c.q('computer', 'Компьютер', 0, 0, [
    'CC: Tweaked приносит настоящие программируемые компьютеры на Lua.'
], ['computercraft:computer_normal'], [('xp', 10)], shape='gear', size=1.5)

c.q('turtle', 'Черепашка', 2, 0, [
    'Робот, который копает, строит и ходит по программе.'
], ['computercraft:turtle_normal'], [('xp', 20)], deps=['computer'])

c.q('modem', 'Сеть', 2, 2, [
    'Модемы связывают компьютеры и периферию в сеть.'
], [('computercraft:wireless_modem_normal', 2)], [('xp', 10)], deps=['computer'])

c.q('cbc_cc', 'Компьютерная артиллерия', 4, 0, [
    'Мод CC:CBC позволяет наводить пушки Big Cannons из программы.',
    'Полностью автоматическая батарея — вполне реально.'
], [{'type': 'checkmark', 'title': 'Подключить пушку к компьютеру'}], [('xp', 30)], deps=['turtle', 'modem'], optional=True)


# --- 13. ТЯЖЁЛАЯ АРТИЛЛЕРИЯ (BIG CANNONS) ---
c = add_chapter('big_cannons', 'Тяжёлая артиллерия', 'createbigcannons:bronze_cannon_barrel', g_war, 'Орудия, лафеты и станки')
c.q('drill', 'Орудийный станок', 0, 0, [
    'Растачивает литые орудийные стволы. Без него пушку не собрать.'
], ['createbigcannons:cannon_drill'], [('item', 'create:andesite_alloy', 8)], shape='gear', size=1.5)

c.q('mount', 'Орудийный лафет', 2, -2, [
    'Удерживает пушку, наводит её по вертикали и горизонтали.'
], ['createbigcannons:cannon_mount'], [('item', 'create:iron_sheet', 8)], deps=['drill'])

c.q('bronze_cannon', 'Бронзовое орудие', 2, 0, [
    'Самая простая пушка: ствол, казённая часть и затвор.'
], [('createbigcannons:bronze_cannon_barrel', 2), 'createbigcannons:bronze_sliding_breech'], [('xp', 15)], deps=['drill'], shape='diamond')

c.q('cast_iron_cannon', 'Чугунное орудие', 4, 0, [
    'Более прочное орудие. Держит увеличенный пороховой заряд.'
], ['createbigcannons:cast_iron_sliding_breech'], [('xp', 25)], deps=['bronze_cannon'])

c.q('steel_cannon', 'Стальное орудие', 6, 0, [
    'Тяжёлая стальная пушка. Максимальная дальность и мощность.'
], ['createbigcannons:built_up_steel_cannon_barrel'], [('xp', 40)], deps=['cast_iron_cannon'], shape='hexagon', size=1.5)

c.q('autocannon', 'Автопушка', 4, 2, [
    'Скорострельная автоматическая пушка малокалиберными снарядами.'
], ['createbigcannons:bronze_autocannon_breech', 'createbigcannons:bronze_autocannon_barrel'], [('xp', 30)], deps=['bronze_cannon'])

c.q('powder', 'Пороховой заряд', 2, 2, [
    'Толкает снаряд по стволу. Больше зарядов — выше дальность!'
], [('createbigcannons:powder_charge', 4)], [('item', 'minecraft:gunpowder', 8)], deps=['drill'])


# --- 14. СНАРЯДЫ И ПВО ---
c = add_chapter('cbc_munitions', 'Снаряды и ПВО', 'createbigcannons:he_shell', g_war, 'Боеприпасы и ракетные комплексы')
c.q('he_shell', 'Фугасный снаряд', 0, 0, [
    'Взрывается при ударе. Разрушает укрепления и наносит огромный урон.'
], ['createbigcannons:he_shell'], [('item', 'minecraft:gunpowder', 16)], shape='gear', size=1.5)

c.q('ap_shell', 'Бронебойный снаряд', 2, 0, [
    'Пробивает броневые блоки перед взрывом.'
], ['createbigcannons:ap_shell'], [('xp', 15)], deps=['he_shell'])

c.q('shrapnel', 'Шрапнель', 2, 2, [
    'Осыпает область стальной дробью. Идеально против пехоты.'
], ['createbigcannons:shrapnel_shell'], [('xp', 15)], deps=['he_shell'])

c.q('mk_shells', 'Усиленные снаряды MK1-MK5', 4, 0, [
    'CBC Enhanced Shells добавляет модернизированные снаряды 5 уровней.',
    'Снаряды MK5 требуют усиленного алмаза и стального механизма.'
], ['cbc_enhanced_shells:ap_shell_mk1'], [('xp', 30)], deps=['ap_shell'], shape='diamond')

c.q('at_rockets', 'Противотанковые ракеты', 4, 2, [
    'CBC AT добавляет кумулятивные ракеты и пусковые рельсы.'
], ['cbc_at:ap_rocket_item'], [('xp', 25)], deps=['ap_shell'])

c.q('mortar', 'Миномётный снаряд', 0, 2, [
    'Навесная стрельба по закрытым позициям.'
], ['createbigcannons:drop_mortar_shell'], [('xp', 10)], deps=['he_shell'], optional=True)


# --- 15. КУЛИНАРИЯ И ФЕРМЕРСТВО ---
c = add_chapter('farming_cooking', 'Кулинария и фермерство', 'farmersdelight:cooking_pot', g_food, 'Вкусная еда и сытный обед')
c.q('knife', 'Разделочный нож', 0, 0, [
    'Нож режет туши, солому и ингредиенты на разделочной доске.'
], ['farmersdelight:iron_knife'], [('item', 'farmersdelight:cutting_board', 1)], shape='gear', size=1.5)

c.q('board', 'Разделочная доска', 2, -2, [
    'Главное место подготовки продуктов.'
], ['farmersdelight:cutting_board'], [('xp', 5)], deps=['knife'])

c.q('stove', 'Кухонная плита', 2, 0, [
    'Плита жарит еду и греет кастрюли с сковородами.'
], ['farmersdelight:stove'], [('item', 'minecraft:coal', 8)], deps=['knife'])

c.q('pot', 'Кастрюля', 4, 0, [
    'Готовит супы, рагу и сложные блюда из нескольких ингредиентов.'
], ['farmersdelight:cooking_pot'], [('xp', 10)], deps=['stove'], shape='diamond')

c.q('skillet', 'Сковорода', 4, 2, [
    'Быстрая обжарка стейков и омлетов.'
], ['farmersdelight:skillet'], [('xp', 5)], deps=['stove'])

c.q('extra_oven', 'Пекарская печь', 6, 0, [
    'Extra Delight добавляет духовку, выпечку и десерты.'
], ['extradelight:oven'], [('item', 'extradelight:flour', 8)], deps=['pot'])

c.q('feast', 'Праздничный пир', 6, 2, [
    'Приготовь сытное блюдо для всей команды!'
], ['farmersdelight:steak_and_potatoes'], [('xp', 20)], deps=['pot'], shape='hexagon')


# --- 16. НАПИТКИ И АЛКОГОЛЬ ---
c = add_chapter('beverages', 'Напитки и алкоголь', 'brewinandchewin:keg', g_food, 'Пивоварение и барное дело')
c.q('keg', 'Ферментационная бочка', 0, 0, [
    'Brewin and Chewin позволяет варить эль, пиво и медовуху.'
], ['brewinandchewin:keg'], [('item', 'minecraft:apple', 8)], shape='gear', size=1.5)

c.q('beer', 'Кружка пива', 2, 0, [
    'Сваренное пиво даёт полезные эффекты.'
], ['brewinandchewin:beer'], [('xp', 10)], deps=['keg'])

c.q('mead', 'Медовуха', 2, 2, [
    'Медовый напиток из сот и яблок.'
], ['brewinandchewin:mead'], [('xp', 10)], deps=['keg'])

c.q('cocktails', 'Коктейльный бар', 4, 0, [
    'Cocktails Delight добавляет соки, джин, мартини и фруктовые напитки.'
], ['cocktailsdelight:gimlet'], [('xp', 15)], deps=['beer'])


# --- 17. РЫБАЛКА И ОКЕАН ---
c = add_chapter('aquaculture', 'Рыбалка и океан', 'aquaculture:neptunium_fishing_rod', g_food, 'Подводные сокровища и удочки')
c.q('rod', 'Улучшенная удочка', 0, 0, [
    'Aquaculture 2 добавляет металлические удочки, наживки и лески.'
], ['aquaculture:iron_fishing_rod'], [('item', 'aquaculture:worm', 8)], shape='gear', size=1.5)

c.q('tackle', 'Ящик для снастей', 2, 0, [
    'Хранит крючки, наживки и заменяет леску.'
], ['aquaculture:tackle_box'], [('xp', 5)], deps=['rod'])

c.q('fillet', 'Филейный нож', 2, 2, [
    'Разделывает улов на рыбу и ценные материалы.'
], ['aquaculture:iron_fillet_knife'], [('xp', 5)], deps=['rod'])

c.q('neptunium', 'Слиток Нептуния', 4, 0, [
    'Редчайший подводный металл из сокровищ.'
], ['aquaculture:neptunium_ingot'], [('xp', 30)], deps=['tackle'], shape='diamond')

c.q('nept_rod', 'Нептуниевая удочка', 6, 0, [
    'Вершина рыболовного снаряжения!'
], ['aquaculture:neptunium_fishing_rod'], [('xp', 50)], deps=['neptunium'], shape='hexagon', size=1.5)


# --- 18. ТОРГОВЛЯ И ЭКОНОМИКА ---
c = add_chapter('economy', 'Торговля и экономика', 'numismatics:bank_terminal', g_misc, 'Банки, монеты и животноводство')
c.q('bank', 'Банковский терминал', 0, 0, [
    'Numismatics позволяет открывать банковские счета и переводить монеты.'
], ['numismatics:bank_terminal'], [('xp', 10)], shape='gear', size=1.5)

c.q('depositor', 'Депозитор', 2, 0, [
    'Принимает монеты за товары или выдаёт сдачи.'
], ['numismatics:brass_depositor'], [('xp', 15)], deps=['bank'])

c.q('easy_trader', 'Житель в блоке', 4, 0, [
    'Easy Villagers позволяет упаковать жителя в удобный авто-терминал.'
], ['easy_villagers:trader'], [('xp', 20)], deps=['depositor'], shape='diamond')

c.q('iron_farm', 'Компактная ферма железа', 6, 0, [
    'Одноблочная ферма железа с големом и зомби.'
], ['easy_villagers:iron_farm'], [('xp', 30)], deps=['easy_trader'], shape='hexagon')


# --- 19. МЕДИА, ФОТОГРАФИЯ И ДЕКОР ---
c = add_chapter('media_decor', 'Фотография и декор', 'exposure:camera', g_misc, 'Камеры, снимки, радио и декор')
c.q('camera', 'Камера', 0, 0, [
    'Exposure позволяет делать настоящие фотографии мира Minecraft!'
], ['exposure:camera'], [('item', 'exposure:black_and_white_film', 1)], shape='gear', size=1.5)

c.q('album', 'Фотоальбом', 2, 0, [
    'Сохраняй снимки приключений в памятный альбом.'
], ['exposure:album'], [('xp', 10)], deps=['camera'])

c.q('etched', 'Собственная музыка', 0, 2, [
    'Etched позволяет записывать любые треки на чистые пластинки!'
], ['etched:etching_table'], [('item', 'etched:blank_music_disc', 2)])

c.q('waterframes', 'Телевизор в игре', 2, 2, [
    'WaterFrames воспроизводит любые видео в мире Minecraft.'
], ['waterframes:tv'], [('xp', 15)], deps=['etched'])

c.q('createdeco', 'Инженерный декор', 4, 0, [
    'Create Deco & Display Delight добавляют сотни красивых блоков.'
], ['createdeco:blue_brass_lamp'], [('xp', 5)], optional=True)


# --- 20. УТИЛИТЫ И МОБОФЕРМЫ (FULL TINY MOB FARM CHAIN) ---
c = add_chapter('utilities', 'Утилиты и мобофермы', 'tinymobfarm:wood_farm', g_misc, 'Компактные фермы и защита')

c.q('lasso', 'Лассо для мобов', 0, 0, [
    'Лассо позволяет поймать моба в инвентарь для установки в ферму.'
], ['tinymobfarm:lasso'], [('xp', 5)], shape='gear', size=1.5)

c.q('wood_farm', '1. Деревянная мобоферма', 2, 0, [
    'Первый уровень Tiny Mob Farm. Медленная генерирация лута моба.'
], ['tinymobfarm:wood_farm'], [('xp', 5)], deps=['lasso'])

c.q('stone_farm', '2. Каменная мобоферма', 4, 0, [
    'Второй уровень: работает в 2 раза быстрее деревянной.'
], ['tinymobfarm:stone_farm'], [('xp', 10)], deps=['wood_farm'])

c.q('iron_farm', '3. Железная мобоферма', 6, 0, [
    'Третий уровень: повышенная скорость притока лута.'
], ['tinymobfarm:iron_farm'], [('xp', 15)], deps=['stone_farm'])

c.q('gold_farm', '4. Золотая мобоферма', 8, 0, [
    'Четвёртый уровень: высокая скорость работы.'
], ['tinymobfarm:gold_farm'], [('xp', 20)], deps=['iron_farm'], shape='diamond')

c.q('diamond_farm', '5. Алмазная мобоферма', 10, 0, [
    'Пятый уровень: сверхбыстрая генерирация лута.'
], ['tinymobfarm:diamond_farm'], [('xp', 30)], deps=['gold_farm'])

c.q('emerald_farm', '6. Изумрудная мобоферма', 12, 0, [
    'Шестой уровень: элитная производительность.'
], ['tinymobfarm:emerald_farm'], [('xp', 40)], deps=['diamond_farm'])

c.q('inferno_farm', '7. Инфернальная мобоферма', 14, 0, [
    'Седьмой уровень: работает с безумной скоростью.'
], ['tinymobfarm:inferno_farm'], [('xp', 50)], deps=['emerald_farm'])

c.q('ultimate_farm', '8. Абсолютная мобоферма', 16, 0, [
    'Восьмой, высший уровень: мгновенная генерирация любого лута моба!'
], ['tinymobfarm:ultimate_farm'], [('xp', 100)], deps=['inferno_farm'], shape='hexagon', size=1.5)

c.q('gravestone', 'Надгробие', 0, 2, [
    'При смерти твои вещи сохраняются в надгробии на месте гибели.'
], [{'type': 'checkmark', 'title': 'Знать, где искать вещи после смерти'}], [('xp', 5)], optional=True)


# ==================== СБОРКА И ЗАПИСЬ ====================

def main():
    print(f"Target directory: {TARGET_DIR}")
    os.makedirs(os.path.join(TARGET_DIR, 'chapters'), exist_ok=True)

    # 1. Запись data.snbt
    data_snbt = """{
\tversion: 13
\ttitle: "SPCreate 2.1"
\ticon: "create:cogwheel"
\tdefault_reward_team: false
\tdefault_consume_items: false
\tdefault_autoclaim_rewards: "disabled"
\tdefault_quest_shape: "circle"
\tdefault_quest_size: 1.0d
\tdrop_loot_crates: false
\tloot_crate_no_drop: {
\t\tpassive: 0
\t\tmonster: 0
\t\tboss: 0
\t}
\tdisable_gui: false
\tgrid_scale: 0.5d
\tpause_game: false
\tlock_message: ""
\tprogression_mode: "linear"
}
"""
    with open(os.path.join(TARGET_DIR, 'data.snbt'), 'w', encoding='utf-8') as f:
        f.write(data_snbt)

    # 2. Запись chapter_groups.snbt
    cg_lines = ["{", "\tchapter_groups: ["]
    for idx, (gid, gtitle) in enumerate(GROUPS):
        comma = "," if idx < len(GROUPS) - 1 else ""
        cg_lines.append(f'\t\t{{ id: "{gid}", title: "{esc(gtitle)}" }}{comma}')
    cg_lines.append("\t]")
    cg_lines.append("}")
    with open(os.path.join(TARGET_DIR, 'chapter_groups.snbt'), 'w', encoding='utf-8') as f:
        f.write('\n'.join(cg_lines) + '\n')

    # 3. Запись глав
    total_quests = 0
    for order_idx, ch in enumerate(CHAPTERS):
        out_path = os.path.join(TARGET_DIR, 'chapters', f"{ch.fname}.snbt")
        rendered = ch.render(order_idx)
        with open(out_path, 'w', encoding='utf-8') as f:
            f.write(rendered)
        total_quests += len(ch.quests)
        print(f"  [OK] Chapter '{ch.fname}' ({ch.title}) -> {len(ch.quests)} quests")

    print(f"\n[DONE] Successfully generated {len(CHAPTERS)} chapters with {total_quests} quests in total!")

if __name__ == '__main__':
    main()
