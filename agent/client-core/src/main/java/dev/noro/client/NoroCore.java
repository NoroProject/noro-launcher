package dev.noro.client;

import dev.noro.client.link.Bridge;
import dev.noro.client.link.Feature;
import dev.noro.client.rules.RuleBook;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import net.minecraft.client.Minecraft;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.fml.common.Mod;
import net.neoforged.neoforge.client.event.ClientPlayerNetworkEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Core of the Noro client mods: the launcher channel and the shared look.
 *
 * <p>The core shows nothing on its own. It holds one connection to the launcher,
 * hands frames to features, and gives them screens, buttons and badges so
 * "Noro-styled" isn't rewritten in every mod.
 *
 * <p>Separate mods sit on top: {@code noro_staff} for moderator tools,
 * {@code noro_player} for the rest. Staff is handed out by entitlement and isn't
 * on every machine — a player must not get half the case panel with an ordinary
 * build.
 *
 * <p>Neither mod carries authority. Clients get patched and jars get decompiled,
 * so any check in here is decoration. The master answers 403 and the screen shows
 * the refusal; that's the only line.
 */
@Mod(value = NoroCore.ID, dist = Dist.CLIENT)
@EventBusSubscriber(modid = NoroCore.ID, value = Dist.CLIENT)
public final class NoroCore {

    public static final String ID = "noro_core";
    public static final Logger LOG = LoggerFactory.getLogger("NoroCore");

    /**
     * Copy-on-write because registration runs on the mod loading thread while
     * frames are dispatched from the socket thread.
     */
    private static final List<Feature> FEATURES = new CopyOnWriteArrayList<>();

    /**
     * The core owns the rule book: a moderator needs it to issue a punishment and
     * a player needs it to read what they were punished for. A copy per mod would
     * eventually show two different rule books.
     */
    private static final RuleBook RULES = new RuleBook();

    private static final Bridge BRIDGE = new Bridge(FEATURES);

    public NoroCore() {}

    static {
        FEATURES.add(RULES);
    }

    public static RuleBook rules() {
        return RULES;
    }

    /** Called from a mod's constructor. */
    public static void register(Feature feature) {
        FEATURES.add(feature);
        LOG.info("feature registered: {}", feature.id());
    }

    public static Bridge bridge() {
        return BRIDGE;
    }

    /** By login the handshake file is guaranteed to be in place. */
    @SubscribeEvent
    public static void onJoin(ClientPlayerNetworkEvent.LoggingIn event) {
        BRIDGE.connect(Minecraft.getInstance().gameDirectory.toPath());
    }

    @SubscribeEvent
    public static void onLeave(ClientPlayerNetworkEvent.LoggingOut event) {
        BRIDGE.close();
    }
}
