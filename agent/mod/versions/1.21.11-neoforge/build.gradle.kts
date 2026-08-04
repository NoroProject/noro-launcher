plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.21.11")
    mappings(loom.officialMojangMappings())

    neoForge("net.neoforged:neoforge:21.11.45")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
