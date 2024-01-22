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
    fun createInvoice(name: String): String
}


@ActivityInterface
interface InvoiceCreationActivities {

    fun createInvoiceDetailsInDb();
    fun performEInvoicing();
    fun generateAndStorePdfInS3();
    fun storeDetailsInDb();
}


class InvoiceCreationActivitiesImpl : InvoiceCreationActivities {
    override fun createInvoiceDetailsInDb() {
        println("createInvoiceDetailsInDb")
    }

    override fun performEInvoicing() {
        println("performEInvoicing")
    }

    override fun generateAndStorePdfInS3() {
        println("generateAndStorePdfInS3")
    }

    override fun storeDetailsInDb() {
        println("storeDetailsInDb")
    }

}

class InvoiceCreationWorkflowImpl : InvoiceCreationWorkflow {
    private val activities: InvoiceCreationActivities = Workflow.newActivityStub(
        InvoiceCreationActivities::class.java,
        ActivityOptions.newBuilder()
            .setStartToCloseTimeout(Duration.ofSeconds(120))
            .build()
    )

    override fun createInvoice(name: String): String {
        activities.createInvoiceDetailsInDb()
        activities.performEInvoicing()
        activities.generateAndStorePdfInS3()
        activities.storeDetailsInDb()
        return "created"
    }

}