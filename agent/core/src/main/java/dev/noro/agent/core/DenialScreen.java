package dev.noro.agent.core;

/**
 * Экран, который видит непущенный игрок.
 */
public final class DenialScreen {

    /**
     * Единственный текст, который агент пишет сам при абсолютной недоступности мастера.
     */
    private static final String MASTER_DOWN =
            "Authentication service is unavailable. Try again in a minute.";

    private DenialScreen() {}

    public static String text(AccessGate.Denial denial, MessageTemplates templates, RuleCatalog rules) {
        if (denial == null) {
            return "";
        }
        if (denial.reason() == AccessGate.Reason.MASTER_DOWN) {
            return MASTER_DOWN;
        }
        String template = template(denial.reason(), denial.punishment(), templates);
        if (denial.punishment() == null) {
            return template;
        }
        String title = rules == null ? null : rules.titleFor(denial.punishment().ruleCode());
        return MessageRender.render(
                template, denial.punishment(), denial.playerName(), title, templates.rulesUrl());
    }

    private static String template(
            AccessGate.Reason reason, PunishmentInfo punishment, MessageTemplates templates) {
        switch (reason) {
            case MAINTENANCE:
                return templates.maintenance();
            case NO_ACCOUNT:
                return templates.noAccount();
            case NO_ACCESS:
                return templates.noAccess();
            default:
                break;
        }
        if (punishment != null) {
            return templates.screen(punishment);
        }
        return reason == AccessGate.Reason.SERVER_BAN
                ? templates.serverBanPermanent()
                : templates.banPermanent();
    }
}
