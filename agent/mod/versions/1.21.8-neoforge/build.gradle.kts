plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.21.8")
    mappings(loom.officialMojangMappings())

    neoForge("net.neoforged:neoforge:21.8.54")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
