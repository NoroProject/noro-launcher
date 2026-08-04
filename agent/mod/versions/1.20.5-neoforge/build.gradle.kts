plugins {
    id("gg.essential.multi-version")
}

// У этой ветки NeoForge опубликована только beta — иначе версия осталась бы
// вообще без агента.
dependencies {
    minecraft("com.mojang:minecraft:1.20.5")
    mappings(loom.officialMojangMappings())

    neoForge("net.neoforged:neoforge:20.5.0-beta")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
