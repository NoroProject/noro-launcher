plugins {
    id("gg.essential.multi-version")
}

// MC 26.x необфусцирована, и toolkit подключает loom вариантом no-remap. Там нет
// ни `mappings`, ни `modImplementation` — ремапить нечего, поэтому лоадер и API
// подключаются обычным `implementation`.
dependencies {
    "minecraft"("com.mojang:minecraft:26.2")

    implementation("net.fabricmc:fabric-loader:0.19.3")
    implementation("net.fabricmc.fabric-api:fabric-api:0.156.0+26.2")

    compileOnly("eu.pb4:placeholder-api:3.1.0-beta.1+26.2")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
