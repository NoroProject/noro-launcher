plugins {
    id("gg.essential.multi-version")
}

// У этой ветки NeoForge опубликована только beta — иначе версия осталась бы
// вообще без агента.
dependencies {
    minecraft("com.mojang:minecraft:1.21.6")
    mappings(loom.officialMojangMappings())

    neoForge("net.neoforged:neoforge:21.6.0-beta")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
