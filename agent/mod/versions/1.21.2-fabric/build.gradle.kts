plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.21.2")
    mappings(loom.officialMojangMappings())

    modImplementation("net.fabricmc:fabric-loader:0.19.3")
    modImplementation("net.fabricmc.fabric-api:fabric-api:0.106.1+1.21.2")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
