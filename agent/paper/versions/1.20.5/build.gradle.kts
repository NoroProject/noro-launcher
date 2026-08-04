// POM paper-api 1.20.5 импортирует net.kyori:adventure-bom:4.17.0-SNAPSHOT, а
// этот снапшот удалён и с papermc, и с обоих sonatype. Без него не разбирается
// сам POM paper-api, поэтому ни resolutionStrategy, ни dependencySubstitution не
// помогают — они срабатывают позже разбора метаданных.
//
// Кладём недостающий BOM в локальный репозиторий рядом: это дословно POM релиза
// 4.17.0 с заменённой строкой версии, то есть ровно те же диапазоны, что
// подставились бы из снапшота. Иначе 1.20.5 несобираема в принципе.
repositories {
    maven {
        name = "AdventureBomStub"
        url = uri(layout.projectDirectory.dir("local-repo"))
        content { includeModule("net.kyori", "adventure-bom") }
    }
}

dependencies {
    compileOnly("io.papermc.paper:paper-api:1.20.5-R0.1-SNAPSHOT")
    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
