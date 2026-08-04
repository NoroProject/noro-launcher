// Сборка агента Paper под Minecraft 1.20.6. Исходник общий, отсюда — только
// координата API: всё остальное настраивает paper/build.gradle.kts.
dependencies {
    compileOnly("io.papermc.paper:paper-api:1.20.6-R0.1-SNAPSHOT")
    compileOnly("net.luckperms:api:5.5")
    compileOnly(project(":core"))
}
