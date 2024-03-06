package com.temporal.accounting.plugins

import com.temporal.accounting.workflows.invoicing.InvoicingWorkflowService
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*

fun Application.configureRouting() {
    routing {
        get("/") {
            call.respondText("Hello World!")
        }
        post("/create-invoice") {
            val tenantId = call.request.header("x-acc-tenant-id")
                ?: throw IllegalStateException("x-acc-tenant-id should not be null")
            val userId =
                call.request.header("x-acc-user-id")
                    ?: throw IllegalStateException("x-acc-user-id should not be null")
            val body = call.receiveText()
            val res= InvoicingWorkflowService.createInvoice(tenantId, userId, body)
            call.respondText(res)

        }
    }
}
