package com.temporal.accounting

import com.temporal.accounting.plugins.configureRouting
import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.logging.*
import io.ktor.server.application.*
import io.ktor.server.engine.*
import io.ktor.server.netty.*
val httpClient by lazy{
    HttpClient(CIO){
        expectSuccess = true
        install(Logging)
    }
}
fun main() {
    TemporalConfiguration.start()
    embeddedServer(Netty, port = 8080, host = "0.0.0.0", module = Application::module)
        .start(wait = true)
}

fun Application.module() {
    configureRouting()
}

