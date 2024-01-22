package com.temporal.accounting

import com.temporal.accounting.workflows.invoicing.InvoiceCreationActivitiesImpl
import com.temporal.accounting.workflows.invoicing.InvoiceCreationWorkflow
import com.temporal.accounting.workflows.invoicing.InvoiceCreationWorkflowImpl
import io.temporal.client.WorkflowClient
import io.temporal.client.WorkflowOptions
import io.temporal.serviceclient.WorkflowServiceStubs
import io.temporal.worker.WorkerFactory

object TemporalConfiguration {

    val service by lazy{
        WorkflowServiceStubs.newLocalServiceStubs()
    }

    val client by lazy{
        WorkflowClient.newInstance(service)
    }
    val factory by lazy {
        WorkerFactory.newInstance(client)
    }

    val worker by lazy{
        factory.newWorker("d")
    }

    fun start(){
        worker.registerWorkflowImplementationTypes(InvoiceCreationWorkflowImpl::class.java)
        worker.registerActivitiesImplementations(InvoiceCreationActivitiesImpl())
        factory.start()
    }

}