pluginManagement {
    repositories {
        gradlePluginPortal()
        maven("https://maven.fabricmc.net/") { name = "Fabric" }
        maven("https://maven.neoforged.net/releases") { name = "NeoForged" }
        // Здесь живут и essential-gradle-toolkit, и препроцессор ReplayMod:
        // официальный maven.replaymod.com недоступен, зеркало Essential — да.
        maven("https://repo.essential.gg/repository/maven-public") { name = "Essential" }
        // Транзитивные зависимости их форка loom: architectury-loom, mercury,
        // refmap-remapper, pack200.
        maven("https://maven.architectury.dev/") { name = "Architectury" }
        maven("https://maven.minecraftforge.net/") { name = "Forge" }
        maven("https://libraries.minecraft.net/") { name = "Mojang" }
        mavenCentral()
    }
}

rootProject.name = "noro-agent"

include("core", "wrapper")

/**
 * Подключает каждый каталог из `<container>/versions/` отдельным проектом.
 * Версии добавляются созданием каталога — списка, который надо править руками,
 * нет ни здесь, ни в графе препроцессора.
 */
fun includeVersions(container: String) {
    include(container)
    file("$container/versions")
        .listFiles { f -> f.isDirectory && File(f, "build.gradle.kts").exists() }
        ?.sortedBy { it.name }
        ?.forEach { dir ->
            include("$container:${dir.name}")
            project(":$container:${dir.name}").projectDir = dir
        }
}

// Fabric и NeoForge: имя проекта `<mc>-<loader>` разбирает сам toolkit.
includeVersions("mod")
// Paper: имя проекта — просто версия Minecraft.
includeVersions("paper")
