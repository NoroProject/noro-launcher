plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.18.2")
    mappings(loom.officialMojangMappings())

    forge("net.minecraftforge:forge:1.18.2-40.3.12")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
