plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.20.4")
    mappings(loom.officialMojangMappings())

    neoForge("net.neoforged:neoforge:20.4.251")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
