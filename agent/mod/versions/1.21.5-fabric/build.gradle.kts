plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.21.5")
    mappings(loom.officialMojangMappings())

    modImplementation("net.fabricmc:fabric-loader:0.19.3")
    modImplementation("net.fabricmc.fabric-api:fabric-api:0.128.2+1.21.5")

    modCompileOnly("eu.pb4:placeholder-api:2.6.4+1.21.5")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
