plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.20.1")
    mappings(loom.officialMojangMappings())

    forge("net.minecraftforge:forge:1.20.1-47.4.22")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
