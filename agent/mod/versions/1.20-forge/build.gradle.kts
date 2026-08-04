plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.20")
    mappings(loom.officialMojangMappings())

    forge("net.minecraftforge:forge:1.20-46.0.14")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
