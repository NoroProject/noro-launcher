// Панель разбора под NeoForge 1.21.1 — единственная сборка мода.
plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.21.1")
    mappings(loom.officialMojangMappings())

    neoForge("net.neoforged:neoforge:21.1.248")

    // Gson уже едет с игрой — так же, как его берёт агент.
    compileOnly("com.google.code.gson:gson:2.10.1")
}
