// Сборка агента Paper под Minecraft 26.1.2. Исходник общий, отсюда — только
// координата API: всё остальное настраивает paper/build.gradle.kts.
dependencies {
    compileOnly("io.papermc.paper:paper-api:26.1.2.build.74-stable")
    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
