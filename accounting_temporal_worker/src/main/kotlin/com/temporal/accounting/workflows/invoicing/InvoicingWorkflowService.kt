package com.temporal.accounting.workflows.invoicing

import com.temporal.accounting.TemporalConfiguration
import io.temporal.client.WorkflowOptions


object InvoicingWorkflowService {


    fun createInvoice(tenantId:String,userId:String,invoiceRequest:String):String {
        val workflow =
            TemporalConfiguration.client
                .newWorkflowStub(
                    InvoiceCreationWorkflow::class.java,
                    WorkflowOptions.newBuilder()
                        .setWorkflowId("s")
                        .setTaskQueue("d")
                        .build()
                )

        return workflow.createInvoice(tenantId,userId,invoiceRequest)
    }
}