package com.temporal.accounting.plugins

import com.temporal.accounting.workflows.invoicing.InvoicingWorkflowService
import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*

fun Application.configureRouting() {
    routing {
        get("/") {
            call.respondText("Hello World!")
        }
        get("/create-invoice"){
           val k = InvoicingWorkflowService.createInvoice()
           call.respondText(k)
        }
    }
}
