package dev.noro.client.link;

import com.google.gson.JsonObject;

/**
 * One capability of the mod. Each lives in its own package, declares its own
 * frames, and gets from the transport only the ones it claims. Case review is
 * the first feature, not a privileged one.
 */
public interface Feature {

    /** Short name for logs and settings. */
    String id();

    /** Return {@code true} if this frame was ours and handled; otherwise it moves on to the next feature. */
    boolean accept(String type, JsonObject envelope);

    default void connected(Bridge bridge) {}

    default void disconnected() {}
}
