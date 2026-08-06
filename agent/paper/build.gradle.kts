// Контейнер сборок Paper. Сам jar'а не даёт — его собирают проекты в
// `versions/`, каждый против своего paper-api.
//
// Препроцессор тут не нужен: Bukkit API для тех вызовов, которыми пользуется
// агент, не менялся с 1.18. Расходится только координата paper-api, поэтому
// исходник подключается общим каталогом, а не генерируется.

/** `1.21.1` → `12101`, `26.2` → `260200`. */
fun encodeMc(mc: String): Int {
    val parts = mc.split(".").map { it.toIntOrNull() ?: 0 }
    return parts[0] * 10000 + (parts.getOrNull(1) ?: 0) * 100 + (parts.getOrNull(2) ?: 0)
}

val coreClasses = project(":core").extensions.getByType(SourceSetContainer::class.java)["main"].output
val sharedSrc = layout.projectDirectory.dir("src/main/java")
val sharedResources = layout.projectDirectory.dir("src/main/resources")

subprojects {
    apply(plugin = "java")

    val mcVersion = name
    // Minecraft дважды поднимал требование к Java: 1.20.5 — с 17 на 21,
    // 26.0 — с 21 на 25. paper-api это указывает в метаданных, и сборка под
    // меньшую версию не разрешается вовсе, а не падает при компиляции.
    val javaVersion = when {
        encodeMc(mcVersion) >= encodeMc("26.0") -> 25
        encodeMc(mcVersion) >= encodeMc("1.20.5") -> 21
        else -> 17
    }

    extensions.configure<JavaPluginExtension> {
        toolchain.languageVersion = JavaLanguageVersion.of(javaVersion)
    }

    extensions.configure<SourceSetContainer> {
        named("main") {
            java.setSrcDirs(listOf(sharedSrc))
            resources.setSrcDirs(listOf(sharedResources))
        }
    }

    tasks.withType<JavaCompile>().configureEach {
        options.encoding = "UTF-8"
    }

    // PlaceholderAPI одинаков для всех версий игры и собран под Java 8, поэтому
    // живёт здесь, а не в двадцати шести `versions/*/build.gradle.kts`. Версия
    // намеренно не самая свежая: расширение пользуется только той частью API,
    // которая не менялась, а рантайм на серверах бывает старым.
    dependencies {
        "compileOnly"("me.clip:placeholderapi:2.11.6")
    }

    tasks.named<Jar>("jar") {
        from(coreClasses)
    }

    tasks.named<ProcessResources>("processResources") {
        val parts = mcVersion.split(".")
        val props = mapOf(
            "version" to project.version.toString(),
            // api-version в plugin.yml — минорная линия, а не патч.
            "apiVersion" to if (parts[0] == "1") parts.take(2).joinToString(".") else parts[0],
        )
        inputs.properties(props)
        filesMatching("plugin.yml") { expand(props) }
    }
}
