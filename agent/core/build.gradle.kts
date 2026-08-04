plugins {
    `java-library`
}

dependencies {
    // Всё compileOnly: и Gson, и slf4j, и LuckPerms уже есть в рантайме на всех
    // трёх платформах. Затащить их в jar — получить конфликт классов с хостом.
    compileOnly(libs.luckperms.api)
    compileOnly(libs.gson)
    compileOnly(libs.slf4j.api)

    testImplementation(libs.luckperms.api)
    testImplementation(libs.gson)
    testImplementation(libs.slf4j.api)
    testImplementation(libs.junit.jupiter)
    testRuntimeOnly(libs.junit.platform.launcher)
}

tasks.test {
    useJUnitPlatform()
}
