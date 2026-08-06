plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.20")
    mappings(loom.officialMojangMappings())

    modImplementation("net.fabricmc:fabric-loader:0.19.3")
    modImplementation("net.fabricmc.fabric-api:fabric-api:0.83.0+1.20")

    modCompileOnly("eu.pb4:placeholder-api:2.1.1+1.20")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
