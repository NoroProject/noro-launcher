plugins {
    java
}

allprojects {
    group = "dev.noro"
    version = "0.1.0"

    repositories {
        mavenCentral()
        maven("https://repo.papermc.io/repository/maven-public/") { name = "PaperMC" }
        maven("https://maven.fabricmc.net/") { name = "Fabric" }
        maven("https://maven.neoforged.net/releases") { name = "NeoForged" }
        maven("https://repo.essential.gg/repository/maven-public") { name = "Essential" }
        maven("https://maven.architectury.dev/") { name = "Architectury" }
        maven("https://libraries.minecraft.net/") { name = "Mojang" }
        maven("https://repo.extendedclip.com/releases/") { name = "PlaceholderAPI" }
        // Text Placeholder API: то же, что PlaceholderAPI, но для Fabric.
        maven("https://maven.nucleoid.xyz/") { name = "Nucleoid" }
    }
}

// 17, а не 21: core едет внутрь каждого агента, включая сборки под MC 1.18–1.20.4,
// где сервер работает на Java 17 — байткод 21 там просто не загрузится. Врапперу
// по той же причине: он запускается на той же машине, что и сервер. Records и
// HttpClient в 17 уже есть, ничего переписывать не пришлось.
val javaVersion = "17"

// Только одноверсионные модули. Внутри `:mod` и `:paper` версию Java выбирают
// их собственные скрипты — по версии Minecraft.
configure(listOf(project(":core"), project(":wrapper"))) {
    apply(plugin = "java")

    extensions.configure<JavaPluginExtension> {
        toolchain.languageVersion = JavaLanguageVersion.of(javaVersion)
    }

    tasks.withType<JavaCompile>().configureEach {
        options.encoding = "UTF-8"
    }
}

/**
 * Складывает все собранные агенты под именами, которые ждёт мастер:
 * `{platform}-{mc}.jar`. Готовый каталог остаётся скопировать в
 * `{NORO_DATA_DIR}/agents/`, откуда его раздаёт `GET /api/agent/artifact`.
 */
val collectAgents by tasks.registering(Copy::class) {
    group = "noro"
    description = "Collects every built agent into build/agents under master naming"
    into(layout.buildDirectory.dir("agents"))
    duplicatesStrategy = DuplicatesStrategy.FAIL

    // Имя версионного проекта мода — `<mc>-<loader>`, у мастера порядок обратный.
    project(":mod").subprojects.forEach { version ->
        val (mc, loader) = version.name.split("-", limit = 2)
        // С MC 26.0 игра необфусцирована, loom работает без ремапа, и задачи
        // remapJar у таких проектов нет — итоговый артефакт даёт обычный jar.
        val unobfuscated = (mc.split(".").firstOrNull()?.toIntOrNull() ?: 0) >= 26
        from(version.tasks.named(if (unobfuscated) "jar" else "remapJar")) {
            rename { "$loader-$mc.jar" }
        }
    }

    // У Paper имя проекта — сама версия Minecraft.
    project(":paper").subprojects.forEach { version ->
        from(version.tasks.named("jar")) { rename { "paper-${version.name}.jar" } }
    }

    // Враппер один на все версии, поэтому без версии в имени. Лежит рядом с
    // агентами, чтобы весь каталог копировался в {NORO_DATA_DIR}/agents одной
    // командой и админка видела его тем же списком.
    from(project(":wrapper").tasks.named("jar")) { rename { "wrapper.jar" } }

    // Клиентские моды: ядро, плеер и стафф для всех доступных версий.
    project(":client-core").subprojects.forEach { version ->
        val taskName = if (version.tasks.findByName("remapJar") != null) "remapJar" else "jar"
        from(version.tasks.named(taskName)) { rename { "client-core-${version.name}.jar" } }

        if (project(":client-player").subprojects.isEmpty()) {
            from(project(":client-player").tasks.named("jar")) { rename { "client-player-${version.name}.jar" } }
        }
        if (project(":client-staff").subprojects.isEmpty()) {
            from(project(":client-staff").tasks.named("jar")) { rename { "client-staff-${version.name}.jar" } }
        }
    }

    if (project(":client-player").subprojects.isNotEmpty()) {
        project(":client-player").subprojects.forEach { version ->
            from(version.tasks.named("jar")) { rename { "client-player-${version.name}.jar" } }
        }
    }
    if (project(":client-staff").subprojects.isNotEmpty()) {
        project(":client-staff").subprojects.forEach { version ->
            from(version.tasks.named("jar")) { rename { "client-staff-${version.name}.jar" } }
        }
    }
}
