package com.temporal.accounting.workflows.invoicing

import com.temporal.accounting.clients.AccountingService
import io.temporal.activity.ActivityInterface
import io.temporal.activity.ActivityOptions
import io.temporal.common.RetryOptions
import io.temporal.workflow.Workflow
import io.temporal.workflow.WorkflowInterface
import io.temporal.workflow.WorkflowMethod
import kotlinx.coroutines.runBlocking
import java.time.Duration

@WorkflowInterface
interface InvoiceCreationWorkflow {

    @WorkflowMethod
    fun createInvoice(tenantId: String, userId: String, invoiceRequest: String): String
}


@ActivityInterface
interface InvoiceCreationActivities {
    fun createInvoiceDetailsInDbAndPerformEInvoicing(tenantId: String, userId: String, invoiceRequest: String): String
    fun generateAndStorePdfInS3AndUpdateDetailsInDb(tenantId: String, userId: String, pdfInput: String): String
}


class InvoiceCreationActivitiesImpl : InvoiceCreationActivities {

    override fun createInvoiceDetailsInDbAndPerformEInvoicing(
        tenantId: String,
        userId: String,
        invoiceRequest: String
    ): String = runBlocking {
        val response = AccountingService.createInvoice(tenantId, userId, invoiceRequest)
        response
    }

    override fun generateAndStorePdfInS3AndUpdateDetailsInDb(
        tenantId: String,
        userId: String,
        pdfInput: String
    ): String = runBlocking {
        val response = AccountingService.createInvoicePdfAndUploadToS3AndUpdateDbDetails(tenantId, userId, pdfInput)
        response
    }

}

class InvoiceCreationWorkflowImpl : InvoiceCreationWorkflow {
    private val activities: InvoiceCreationActivities = Workflow.newActivityStub(
        InvoiceCreationActivities::class.java,
        ActivityOptions.newBuilder()
            .setRetryOptions(
                RetryOptions.newBuilder()
                    .setInitialInterval(Duration.ofSeconds(100))
                    .build()
            )

            .setStartToCloseTimeout(Duration.ofSeconds(100))
            .build()
    )

//input will be json
//if already created then do nothing i guess?. to find if everything done or not call createInvoiceDetailsIndb and in response
//return if all complete? why would it be incomplete? we are only creating invoice from this workflow
//this means inflight requests have been taken care of.
//all steps will need to be idempotent then
    override fun createInvoice(tenantId: String, userId: String, invoiceRequest: String): String {
        val output = activities.createInvoiceDetailsInDbAndPerformEInvoicing(tenantId, userId, invoiceRequest)
        val presignedInvoiceUrl = activities.generateAndStorePdfInS3AndUpdateDetailsInDb(tenantId, userId, output)
        return presignedInvoiceUrl
    }

}