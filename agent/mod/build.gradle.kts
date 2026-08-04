import com.replaymod.gradle.preprocess.Node

plugins {
    id("gg.essential.multi-version.root") version "0.7.2"
}

/** `1.21.1` → `12101`, `26.2` → `260200`. Порядок сохраняется, сравнения в `//#if MC>=…` работают. */
fun encodeMc(mc: String): Int {
    val parts = mc.split(".").map { it.toIntOrNull() ?: 0 }
    return parts[0] * 10000 + (parts.getOrNull(1) ?: 0) * 100 + (parts.getOrNull(2) ?: 0)
}

// Граф строится из самих каталогов `versions/`, а не перечислением: добавить
// версию — значит создать папку `<mc>-<loader>`, и всё. Иначе список пришлось бы
// править в трёх местах на каждую из четырёх десятков сборок.
//
// Исходник живёт один — в `mod/src/main/java`, он же mainProject. Мэппинги у всех
// узлов `official` (Mojang), поэтому препроцессору не надо переименовывать
// идентификаторы между yarn и mojmap — остаётся только работа с `//#if`.
preprocess {
    val versions = file("versions")
        .listFiles { f -> f.isDirectory && File(f, "build.gradle.kts").exists() }
        .orEmpty()
        .map { dir -> dir.name.split("-", limit = 2) }
        .map { (mc, loader) -> Triple(mc, loader, encodeMc(mc)) }
        .sortedBy { it.third }

    // Fabric — становой хребет графа: он единственный покрывает весь диапазон,
    // поэтому цепочку версий строим по нему.
    val fabricByMc = LinkedHashMap<String, Node>()
    var previous: Node? = null
    versions.filter { it.second == "fabric" }.forEach { (mc, loader, encoded) ->
        val node = createNode("$mc-$loader", encoded, "official")
        previous?.let { node.link(it) }
        fabricByMc[mc] = node
        previous = node
    }

    // Остальные лоадеры прицепляются к Fabric той же версии игры: между ними
    // расходится только точка входа, а версия Minecraft — общая.
    versions.filterNot { it.second == "fabric" }.forEach { (mc, loader, encoded) ->
        val node = createNode("$mc-$loader", encoded, "official")
        fabricByMc[mc]?.let { node.link(it) }
    }
}

// core вкладывается классами в каждый мод: у него нет runtime-зависимостей,
// поэтому shadow и релокация не нужны. Именно `jar`, а не все задачи типа Jar —
// иначе классы попали бы в remapJar по второму разу.
val coreClasses = project(":core").extensions.getByType(SourceSetContainer::class.java)["main"].output

/**
 * Каталог с метаданными мода для этой связки версия+лоадер.
 *
 * Общего файла тут быть не может: NeoForge до 1.21 читает `META-INF/mods.toml`,
 * как и Forge, но зависит от `neoforge`, а не `forge`, и пишет `type=` вместо
 * `mandatory=`. Один файл на обоих — это NeoForge, который не грузится вовсе,
 * потому что требует несуществующий мод `forge`.
 */
fun metadataFlavour(mc: String, loader: String): String = when {
    loader == "fabric" -> "fabric"
    loader == "forge" -> "forge"
    encodeMc(mc) >= encodeMc("1.21") -> "neoforge"
    else -> "neoforge-legacy"
}

subprojects {
    // Через withId, а не напрямую: задачу `jar` заводит java-плагин, который
    // здесь применяет multi-version, и на момент этого блока её ещё нет.
    plugins.withId("java") {
        val (mc, loader) = name.split("-", limit = 2)
        val flavour = metadataFlavour(mc, loader)

        tasks.named<Jar>("jar") {
            from(coreClasses)
            // Прямо в jar, а не через resources: препроцессор подменяет каталог
            // ресурсов своим (build/preprocessed), и добавленный srcDir туда не
            // доезжает. Препроцессировать эти файлы всё равно нечего — нужна
            // только подстановка версии.
            from(rootProject.file("mod/src/$flavour/resources")) {
                filter { line -> line.replace("\${version}", project.version.toString()) }
            }
        }
    }
}
