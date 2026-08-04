plugins {
    id("gg.essential.multi-version")
}

// MC 26.x необфусцирована, и toolkit подключает loom вариантом no-remap. Там нет
// ни `mappings`, ни `modImplementation` — ремапить нечего, поэтому лоадер и API
// подключаются обычным `implementation`.
dependencies {
    "minecraft"("com.mojang:minecraft:26.1.1")

    implementation("net.fabricmc:fabric-loader:0.19.3")
    implementation("net.fabricmc.fabric-api:fabric-api:0.145.4+26.1.1")

    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
