// Ядро клиентских модов: канал с лаунчером и общий вид.
//
// Матрицы версий здесь нет — клиент один, ровно под опубликованную сборку.
// Контейнер `versions/` с единственным узлом всё же взят: он даёт тот же loom,
// что у агента. `//#if` в исходнике при этом не появляется.

plugins {
    id("gg.essential.multi-version.root") version "0.7.2"
}

preprocess {
    createNode("1.21.1-neoforge", 12101, "official")
}

subprojects {
    plugins.withId("java") {
        // Через `base.archivesName`: loom кладёт готовый jar задачей `remapJar`,
        // и переименовывать пришлось бы обе задачи.
        extensions.configure<BasePluginExtension> { archivesName.set("noro-core") }
    }
}
