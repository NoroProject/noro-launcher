plugins {
    id("gg.essential.multi-version")
}

dependencies {
    minecraft("com.mojang:minecraft:1.21.1")
    mappings(loom.officialMojangMappings())

    modImplementation("net.fabricmc:fabric-loader:0.19.3")
    modImplementation("net.fabricmc.fabric-api:fabric-api:0.116.15+1.21.1")

    modCompileOnly("eu.pb4:placeholder-api:2.4.2+1.21")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
