plugins {
    application
}

dependencies {
    implementation(project(":core"))
    implementation(libs.gson)
    implementation(libs.slf4j.api)
    runtimeOnly(libs.slf4j.simple)

    testImplementation(libs.gson)
    testImplementation(libs.junit.jupiter)
    testRuntimeOnly(libs.junit.platform.launcher)
}

application {
    mainClass = "dev.noro.agent.wrapper.Main"
}

// Враппер запускается как самостоятельный процесс до сервера, поэтому classpath
// ему собрать неоткуда — всё едет в одном jar.
tasks.jar {
    manifest { attributes("Main-Class" to "dev.noro.agent.wrapper.Main") }
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    // zipTree разворачивает архивы и теряет связь с задачей, которая их собрала,
    // поэтому зависимость от runtimeClasspath объявляем руками.
    dependsOn(configurations.runtimeClasspath)
    from(configurations.runtimeClasspath.get().map { if (it.isDirectory) it else zipTree(it) }) {
        exclude("META-INF/*.SF", "META-INF/*.DSA", "META-INF/*.RSA", "module-info.class")
    }
}

tasks.test {
    useJUnitPlatform()
}
