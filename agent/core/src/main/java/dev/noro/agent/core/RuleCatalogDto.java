package dev.noro.agent.core;

import java.util.List;

/**
 * DTO ответа {@code GET /api/agent/rules}.
 */
public final class RuleCatalogDto {

    private RuleCatalogDto() {}

    public static final class RulesData {
        public List<Category> categories;
        public List<Rule> rules;
        public List<Sanction> sanctions;
    }

    public static final class Category {
        public String id;
        public String name;
        public String description;
    }

    public static final class Rule {
        public String id;
        public String categoryId;
        public String code;
        public String title;
        public String description;
        public String punishReason;
    }

    public static final class Sanction {
        public String ruleId;
        public String kind;
        public Long minMinutes;
        public Long maxMinutes;
    }
}
