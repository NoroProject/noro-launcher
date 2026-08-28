/** The rulebook: categories, rules, and the sanctions allowed for each. */

import type { InjectionKey } from 'vue'

export type PunishmentKind = 'warn' | 'mute' | 'ban' | 'server_ban'

export interface RuleCategory {
  id: string;
  parent_id: string | null;
  /** `null` for the shared rulebook, otherwise an addition for one server. */
  server_id: string | null;
  /** Category number: "1", "2.3". Rule codes are derived from it. */
  code: string;
  name: string;
  description: string;
  sort_order: number;
}

export interface Rule {
  id: string;
  category_id: string | null;
  server_id: string | null;
  /** Short code, quoted in chat, bans and tickets. */
  code: string;
  title: string;
  description: string;
  /** The reason a player sees when punished under this rule. */
  punish_reason: string;
  sort_order: number;
}

/**
 * A sanction allowed for a rule: kind plus duration bounds. It's a range, not
 * a single number — a moderator picks inside it, and only
 * `noro.mod.punish.bypass` gets out of it.
 */
export interface RuleSanction {
  id: string;
  rule_id: string;
  kind: PunishmentKind;
  /** Which case this variant covers: first offence, repeat, and so on. */
  label: string;
  /** `null` means no lower bound. */
  min_minutes: number | null;
  /** `null` means anything up to permanent. */
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
 * Title and body in one language. The base-language entry is the rule's or
 * category's own fields; the rest live in `translations`.
 */
export interface LocalizedText {
  locale: string;
  title: string;
  description: string;
  /** Punishment wording. Rules have it, categories don't. */
  punish_reason?: string;
}

/** A server whose rulebook differs from the shared one. */
export interface RuleScope {
  id: string;
  name: string;
}

export interface RuleNode {
  category: RuleCategory;
  rules: Rule[];
  children: RuleNode[];
  /** Rules here and in every nested category. Empty branches aren't drawn. */
  total: number;
}

/**
 * Actions for the admin tree. The tree is recursive, so the page hands these
 * down through `provide` rather than threading eight handlers per level.
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
  /** Without these the tree still reads fine, it just loses its buttons. */
  canEdit: boolean;
  canDelete: boolean;
}

export const RULE_ACTIONS = Symbol('rule-actions') as InjectionKey<RuleActions>
