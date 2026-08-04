plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.21.3")
    mappings(loom.officialMojangMappings())

    neoForge("net.neoforged:neoforge:21.3.97")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
