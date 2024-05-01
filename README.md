# Accounting System (Experimental)

[![Experimental](https://img.shields.io/badge/status-experimental-orange)](https://github.com/your-repo/accounting-system)

Accounting System is an open-source project that aims to provide a set of performant APIs for handling Indian accounting in businesses easily. It is being designed to cater to the needs of businesses with a volume of monthly orders and invoices that cannot be efficiently managed and tracked using manual processes or traditional ERP systems.

*Please note that this project is currently in an experimental phase and may undergo breaking changes or a complete revamp at any stage.*

## Motivation

With the introduction of e-invoicing in India, businesses are upgrading their invoice stacks to create digital invoices. Also ecommerce operations are generating a huge transaction volume in businesses that needs to be accounted. However, there is a lack of open-source tools for creating B2B or B2C gst compliant invoices. Companies often have to invest heavily in developing their own tech for creating invoices or resort to expensive ERPs. This project aims to fill this gap by providing an open-source solution that is feature-rich, scalable, and easy to integrate.

## Key Features being targeted

- **Multi-tenancy:** Support for multiple organizations as tenants with isolated data and users. (done)
- **Multi-currency:** Ability to register and handle different currencies, including real and virtual currencies.
- **Account Management:** Create multiple accounts and define account hierarchies.
- **Ledger Maintenance:** Maintain a ledger for financial transactions.
- **Invoice Creation:** Generate different types of invoices and their PDFs.

## High-Level Design

![accounting_system_image](https://github.com/AayushyaVajpayee/accounting-system/assets/24789440/baa1a4b1-f703-46a9-a8d5-f78208d08587)

The high-level design of the Accounting System consists of the following components:

- **User Clients:** The users consuming the accounting APIs to run their business.
- **API Gateway:** The entry point for APIs, handling authentication using third party providers.(this is planned but not yet integrated)
- **Accounting System Service:** The main logic resides here, handling requests from users via the API Gateway.
- **Temporal Cluster:** An open-source service for durable execution of workflows as code, deployed using a customized Helm chart.
- **Temporal Worker:** A Kotlin service that executes the actual workflows and interacts with other services.
- **PDF Generator:** A separate service for generating PDFs, isolated from the accounting service to handle compute and memory-intensive tasks.
- **PostgreSQL Database:** The database for storing accounting data, managed as a service.
- **Storage:** S3 storage for storing generated PDFs and other files, managed as a service.

## Getting Started
