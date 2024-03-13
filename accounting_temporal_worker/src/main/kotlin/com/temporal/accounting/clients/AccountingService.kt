package com.temporal.accounting.clients

import com.temporal.accounting.httpClient
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*

object AccountingService {

    private const val KUBERNETES_SERVICE: String = "http://accounting-system"
    private const val LOCAL_SERVICE: String = "http://localhost:8090"//todo test this too
    private const val TENANT_HEADER_KEY = "x-acc-tenant-id"
    private const val USER_ID_HEADER_KEY = "x-acc-user-id"

    private val CREATE_INVOICE_API by lazy {
        val domain = getPrefixDomain()
        "$domain/invoice/create"
    }
    private val CREATE_INVOICE_PDF by lazy {
        val domain = getPrefixDomain()
        "$domain/invoice/create-pdf"
    }


    fun getPrefixDomain(): String {
        val isKubernetesEnv: String? = System.getenv("IS_KUBERNETES_ENV")
        return if (isKubernetesEnv.isNullOrBlank() || isKubernetesEnv.toBoolean().not()) {
            LOCAL_SERVICE
        } else {
            KUBERNETES_SERVICE
        }
    }
    private fun setDefaultHeaders(headers: HeadersBuilder,tenantId: String,userId: String) {
        headers.apply {
            append(TENANT_HEADER_KEY, tenantId)
            append(USER_ID_HEADER_KEY, userId)
        }
    }

    suspend fun createInvoice(tenantId: String, userId: String, invoiceRequest: String): String {
        val httpCall = httpClient.post(CREATE_INVOICE_API) {
            setDefaultHeaders(this.headers,tenantId,userId)
            contentType(ContentType.Application.Json)
            setBody(invoiceRequest)
        }.call
        return httpCall.response.bodyAsText()
    }

    suspend fun createInvoicePdfAndUploadToS3AndUpdateDbDetails(
        tenantId: String,
        userId: String,
        pdfInput: String
    ): String {
        val httpCall = httpClient.post(CREATE_INVOICE_PDF) {
            setDefaultHeaders(this.headers,tenantId,userId)
            contentType(ContentType.Application.Json)
            setBody(pdfInput)
        }.call
        return httpCall.response.bodyAsText()
    }


}