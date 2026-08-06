plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.20.1")
    mappings(loom.officialMojangMappings())

    modImplementation("net.fabricmc:fabric-loader:0.19.3")
    modImplementation("net.fabricmc.fabric-api:fabric-api:0.92.11+1.20.1")

    modCompileOnly("eu.pb4:placeholder-api:2.1.4+1.20.1")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
