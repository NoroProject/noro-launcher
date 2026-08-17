/** Свод правил: разделы, правила и допустимые за них наказания. */

import type { InjectionKey } from 'vue'

export type PunishmentKind = 'warn' | 'mute' | 'ban' | 'server_ban'

export interface RuleCategory {
  id: string;
  parent_id: string | null;
  /** `null` — раздел общего свода, иначе дополнение конкретного сервера. */
  server_id: string | null;
  /** Номер раздела: «1», «2.3». Из него выводятся коды правил. */
  code: string;
  name: string;
  description: string;
  sort_order: number;
}

export interface Rule {
  id: string;
  category_id: string | null;
  server_id: string | null;
  /** Короткий код: им ссылаются в чате, банах и тикетах. */
  code: string;
  title: string;
  description: string;
  sort_order: number;
}

/**
 * Допустимое наказание за правило: вид и рамки срока. Вилка, а не одно число —
 * модератор выбирает внутри неё, а выйти за неё может только тот, кому выдан
 * `noro.mod.punish.bypass`.
 */
export interface RuleSanction {
  id: string;
  rule_id: string;
  kind: PunishmentKind;
  /** Пояснение к варианту: «первое нарушение», «повторное». */
  label: string;
  /** `null` — без нижней границы. */
  min_minutes: number | null;
  /** `null` — допустимо вплоть до «навсегда». */
  max_minutes: number | null;
  sort_order: number;
}

export interface RulesResponse {
  server_id: string | null;
  categories: RuleCategory[];
  rules: Rule[];
  sanctions: RuleSanction[];
}

/**
 * Заголовок и текст записи на одном языке. Запись базового языка — это сами
 * поля правила или раздела, остальные уходят в `translations`.
 */
export interface LocalizedText {
  locale: string;
  title: string;
  description: string;
}

/** Сервер, у которого свод отличается от общего. */
export interface RuleScope {
  id: string;
  name: string;
}

/** Раздел вместе со своими правилами и подразделами — то, что рисует страница. */
export interface RuleNode {
  category: RuleCategory;
  rules: Rule[];
  children: RuleNode[];
  /** Правил в этом разделе и во всех вложенных: пустые ветки не рисуются. */
  total: number;
}

/**
 * Действия админского дерева. Дерево рекурсивное, и прокидывать восемь
 * обработчиков через каждый уровень означало бы повторять их на каждом —
 * вместо этого страница выдаёт их через `provide`.
 */
export interface RuleActions {
  editRule: (rule: Rule) => void;
  newRule: (category: RuleCategory | null) => void;
  editCategory: (category: RuleCategory) => void;
  newSection: (parent: RuleCategory | null) => void;
  removeRule: (rule: Rule) => void;
  removeCategory: (category: RuleCategory) => void;
  move: (kind: 'rules' | 'categories', id: string, delta: number) => void;
  serverName: (id: string | null) => string;
  sanctionsOf: (ruleId: string) => RuleSanction[];
  /** Права на свод: без них дерево остаётся читаемым, но кнопки не рисуются. */
  canEdit: boolean;
  canDelete: boolean;
}

export const RULE_ACTIONS = Symbol('rule-actions') as InjectionKey<RuleActions>
