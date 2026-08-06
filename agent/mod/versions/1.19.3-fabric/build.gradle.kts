plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.19.3")
    mappings(loom.officialMojangMappings())

    modImplementation("net.fabricmc:fabric-loader:0.19.3")
    modImplementation("net.fabricmc.fabric-api:fabric-api:0.76.1+1.19.3")

    modCompileOnly("eu.pb4:placeholder-api:2.0.0-rc.1+1.19.3")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
