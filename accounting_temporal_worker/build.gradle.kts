import java.io.FileInputStream
import java.nio.file.Files
import java.util.Properties

val ktor_version: String by project
val kotlin_version: String by project
val logback_version: String by project

plugins {
    kotlin("jvm") version "1.9.22"
    id("io.ktor.plugin") version "2.3.7"
}

group = "com.temporal.accounting"
version = "0.0.1"


application {
    mainClass.set("com.temporal.accounting.ApplicationKt")

    val isDevelopment: Boolean = project.ext.has("development")
    applicationDefaultJvmArgs = listOf("-Dio.ktor.development=$isDevelopment")
}
ktor {

    docker {
        jreVersion.set(JavaVersion.VERSION_21)
        localImageName.set("accounting_temporal_java_worker")
        imageTag.set(providers.environmentVariable("GITHUB_SHA").getOrElse("local_or_unknown"))
        externalRegistry.set(
            io.ktor.plugin.features.DockerImageRegistry.externalRegistry(
                username = provider { "AWS" },
                password = providers.provider {
                    get_value("AWS_PASSWORD")
                },
                hostname = providers.provider {
                    get_value("AWS_ECR_HOSTNAME")
                },
                project = provider { "accounting_temporal_java_worker" },
            )
        )

    }
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("io.ktor:ktor-server-core-jvm")
    implementation("io.ktor:ktor-server-netty-jvm")
    implementation("ch.qos.logback:logback-classic:$logback_version")
    implementation("io.temporal:temporal-sdk:1.22.3")
    testImplementation("io.ktor:ktor-server-tests-jvm")
    testImplementation("org.jetbrains.kotlin:kotlin-test-junit:$kotlin_version")
}

fun get_value(key: String): String {
    return if (providers.environmentVariable("CI").isPresent) {
        providers.environmentVariable("key").get()
    } else {
        val k = Properties();
        k.load(FileInputStream("custom.properties"));
        k[key] as String

    }

}