// Инструменты модератора поверх ядра: панель разбора дел.
//
// Отдельный jar, а не часть ядра: staff раздаётся по праву и лежит не у всех, а
// сверка целостности удаляет чужим лишний файл как ForbiddenOptionalMod.
//
// Обычный java-проект, без loom и препроцессора. Minecraft и NeoForge приезжают
// готовым classpath от ядра: версия здесь одна, выбирать препроцессору нечего, а
// второй контейнер мультиверсий столкнулся бы с первым по имени узла.
// Ремап не нужен — NeoForge 1.21 работает на официальных мэппингах Mojang, и
// имена, с которыми мы компилируемся, и есть боевые.

plugins {
    java
}

java {
    toolchain.languageVersion = JavaLanguageVersion.of(21)
}

// Классы ядра и его classpath (Minecraft, NeoForge) — одной строкой: loom
// собирает их у ядра, и повторять эту сборку здесь незачем.
val coreClasspath: FileCollection = files(
    provider {
        val core = project(":client-core:1.21.1-neoforge")
        val main = core.extensions.getByType(SourceSetContainer::class.java)["main"]
        main.output + main.compileClasspath
    }
)

dependencies {
    compileOnly(coreClasspath)
}

tasks.withType<JavaCompile>().configureEach { options.encoding = "UTF-8" }

tasks.named<Jar>("jar") {
    archiveBaseName.set("noro-staff")
    // Версия в метаданных — из сборки, как у остальных модов.
    filesMatching("META-INF/neoforge.mods.toml") {
        filter { line -> line.replace("\${version}", project.version.toString()) }
    }
}

