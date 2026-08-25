package dev.noro.client.staff;

import com.mojang.blaze3d.platform.InputConstants;
import dev.noro.client.staff.Screenshot;
import dev.noro.client.staff.ui.CaseScreen;
import dev.noro.client.staff.ui.QueueScreen;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.neoforge.client.event.ClientTickEvent;
import net.neoforged.neoforge.client.event.RegisterKeyMappingsEvent;

/**
 * Клавиши и подключение к лаунчеру.
 *
 * <p>Панель открывается только при праве {@code noro.mod.cases.view}: не потому
 * что это защита — её тут нет и быть не может, — а потому что игроку без прав
 * нечего показывать.
 */
@EventBusSubscriber(modid = NoroStaff.ID, value = Dist.CLIENT)
public final class StaffHooks {

    private static final String CATEGORY = "key.categories.noro_staff";

    // Умолчания — функциональный ряд, а не буквы. В сборке на 180 модов занята
    // каждая буква: R уже перезагружает шейдеры, K показывает рецепт, и
    // конфликт клавиш значит, что одно нажатие делает два дела сразу.
    // F3 и F4 не берём: F3 — отладочные сочетания, F4 у macOS системная.

    /** Полноэкранная панель — отдельной клавишей и только когда нужна. */
    public static final KeyMapping OPEN = key("panel", InputConstants.KEY_F8);

    /** Колесо действий: зажал, повёл мышью, отпустил. */
    public static final KeyMapping RADIAL = key("radial", InputConstants.KEY_F9);

    /** Кадр экрана в дело, минуя папку {@code screenshots/}. */
    public static final KeyMapping SHOT = key("shot", InputConstants.KEY_F10);

    private StaffHooks() {}

    private static KeyMapping key(String name, int code) {
        return new KeyMapping(
                "key.noro_staff." + name, InputConstants.Type.KEYSYM, code, CATEGORY);
    }

    @SubscribeEvent
    public static void registerKeys(RegisterKeyMappingsEvent event) {
        event.register(OPEN);
        event.register(RADIAL);
        event.register(SHOT);
    }

    @SubscribeEvent
    public static void onTick(ClientTickEvent.Post event) {
        Minecraft mc = Minecraft.getInstance();
        if (mc.player == null) {
            return;
        }
        while (OPEN.consumeClick()) {
            if (NoroStaff.state().can(NoroStaff.PERM_VIEW)) {
                mc.setScreen(panel());
            }
        }
        while (SHOT.consumeClick()) {
            Screenshot.attachToOpenCase();
        }
        CaseWheel.hold(RADIAL.isDown());
    }

    /**
     * Что показать по клавише: взятое дело, а если его нет — очередь.
     *
     * <p>Всегда открывать очередь было неудобно ровно там, где панель и нужна:
     * посреди разбора её открывают и закрывают десятки раз, и каждый раз
     * приходилось заново искать своё дело в списке. Вернуться к очереди можно
     * кнопкой — обратный путь дешевле, чем поиск.
     */
    private static net.minecraft.client.gui.screens.Screen panel() {
        CaseModels.View open = NoroStaff.state().open();
        return open == null || open.brief() == null
                ? new QueueScreen()
                : new CaseScreen(open.brief().id());
    }
}
