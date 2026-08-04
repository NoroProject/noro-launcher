plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.19")
    mappings(loom.officialMojangMappings())

    forge("net.minecraftforge:forge:1.19-41.1.0")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
