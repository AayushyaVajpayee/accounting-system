package com.temporal.accounting.workflows.invoicing

import io.temporal.activity.ActivityInterface
import io.temporal.activity.ActivityOptions
import io.temporal.workflow.Workflow
import io.temporal.workflow.WorkflowInterface
import io.temporal.workflow.WorkflowMethod
import java.time.Duration

@WorkflowInterface
interface InvoiceCreationWorkflow {

    @WorkflowMethod
    fun createInvoice(tenantId: String, userId: String, invoiceRequest: String): String
}


@ActivityInterface
interface InvoiceCreationActivities {
    fun createInvoiceDetailsInDbAndPerformEInvoicing(tenantId: String, userId: String, invoiceRequest: String): String
    fun generateAndStorePdfInS3AndUpdateDetailsInDb()
}


class InvoiceCreationActivitiesImpl : InvoiceCreationActivities {

    override fun createInvoiceDetailsInDbAndPerformEInvoicing(
        tenantId: String,
        userId: String,
        invoiceRequest: String
    ): String {
        println("userId $userId tenantId $tenantId")
        println(invoiceRequest)
        return "random placeholder for actual data"
    }

    override fun generateAndStorePdfInS3AndUpdateDetailsInDb() {
        println("generateAndStorePdfInS3 and storeDetailsInDb")
    }

}

class InvoiceCreationWorkflowImpl : InvoiceCreationWorkflow {
    private val activities: InvoiceCreationActivities = Workflow.newActivityStub(
        InvoiceCreationActivities::class.java,
        ActivityOptions.newBuilder()
            .setStartToCloseTimeout(Duration.ofSeconds(120))
            .build()
    )

    //input will be json
//if already created then do nothing i guess?. to find if everything done or not call createInvoiceDetailsIndb and in response
//return if all complete? why would it be incomplete? we are only creating invoice from this workflow
//this means inflight requests have been taken care of.
//all steps will need to be idempotent then
    override fun createInvoice(tenantId: String, userId: String, invoiceRequest: String): String {
        activities.createInvoiceDetailsInDbAndPerformEInvoicing(tenantId, userId, invoiceRequest)
        activities.generateAndStorePdfInS3AndUpdateDetailsInDb()

        return "created"
    }

}