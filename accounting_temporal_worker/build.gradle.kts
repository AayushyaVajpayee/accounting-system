val ktor_version: String by project
val kotlin_version: String by project
val logback_version: String by project

plugins {
    kotlin("jvm") version "1.9.23"
    id("io.ktor.plugin") version "2.3.7"
    id("com.google.cloud.tools.jib") version "3.4.0"
    idea
}

group = "com.temporal.accounting"
version = "0.0.1"


application {
    mainClass.set("com.temporal.accounting.ApplicationKt")

    val isDevelopment: Boolean = project.ext.has("development")
    applicationDefaultJvmArgs = listOf("-Dio.ktor.development=$isDevelopment")
}
jib{
    from{
        image="eclipse-temurin:21-jre-jammy"
    }
    to{
        val tag=(System.getenv("IMAGE_TAG")?:"").ifEmpty{ "latest" }
        image="accounting-temporal-java-worker:${tag}"
    }
    container{
        ports= listOf("8080")
    }
}

ktor {
}

repositories {
    mavenCentral()
}
idea {
    module {
        isDownloadJavadoc=true
        isDownloadSources=true
    }
}
dependencies {
    implementation("io.ktor:ktor-server-core-jvm:$ktor_version")
    implementation("io.ktor:ktor-server-netty-jvm:$ktor_version")
    implementation("io.ktor:ktor-client-core:$ktor_version")
    implementation("io.ktor:ktor-client-cio:$ktor_version")
    implementation("io.ktor:ktor-client-logging:$ktor_version")
    implementation("ch.qos.logback:logback-classic:$logback_version")
    implementation("io.temporal:temporal-sdk:1.22.3")
    testImplementation("io.ktor:ktor-server-tests-jvm")
    testImplementation("org.jetbrains.kotlin:kotlin-test-junit:$kotlin_version")
}
