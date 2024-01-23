package com.temporal.accounting

import com.temporal.accounting.workflows.invoicing.InvoiceCreationActivitiesImpl
import com.temporal.accounting.workflows.invoicing.InvoiceCreationWorkflowImpl
import io.grpc.ManagedChannelBuilder
import io.temporal.client.WorkflowClient
import io.temporal.serviceclient.WorkflowServiceStubs
import io.temporal.serviceclient.WorkflowServiceStubsOptions
import io.temporal.worker.WorkerFactory
import java.time.Duration

object TemporalConfiguration {

    val service by lazy{
        WorkflowServiceStubs.newConnectedServiceStubs(WorkflowServiceStubsOptions.newBuilder()
            .setChannel(ManagedChannelBuilder
                .forAddress("accounting-temporal-frontend",7233)//todo take the service name as env variable through kubernetes instead of hardcoding here.
                .usePlaintext().build())

            .build(), Duration.ofSeconds(30))
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