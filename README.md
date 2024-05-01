# Accounting System (Experimental)

[![Experimental](https://img.shields.io/badge/status-experimental-orange)](https://github.com/your-repo/accounting-system)

Accounting System is an open-source project that aims to provide a performant API for handling Indian accounting in businesses easily. It is designed to cater to the needs of companies with a high volume of monthly orders and invoices that cannot be efficiently managed and tracked using manual processes or traditional ERP systems.

*Please note that this project is currently in an experimental phase and may undergo significant changes.*

## Motivation

With the introduction of e-invoicing in India, businesses are upgrading their invoice stacks to create digital invoices. However, there is a lack of open-source tools for creating B2B GST invoices. Companies often have to invest heavily in developing their own tech for creating invoices or resort to expensive ERPs. Accounting System aims to fill this gap by providing an open-source solution that is feature-rich, scalable, and easy to integrate.

## Key Features

- **Multi-tenancy:** Support for multiple organizations as tenants with isolated data and users.
- **Multi-currency:** Ability to register and handle different currencies, including real and virtual currencies.
- **Account Management:** Create multiple accounts and define account hierarchies.
- **Ledger Maintenance:** Maintain a ledger for financial transactions.
- **Invoice Creation:** Generate different types of invoices and their PDFs.
- **GST Compliance:** Compliant with GST regulations in India.

## High-Level Design

![High-Level Design Diagram](https://i.imgur.com/hLD4DiB.png)

The high-level design of the Accounting System consists of the following components:

- **User Clients:** The users consuming the accounting APIs to run their business.
- **API Gateway:** The entry point for APIs, handling authentication using AWS Cognito or Okta Auth0.
- **Accounting System Service:** The main logic resides here, handling requests from users via the API Gateway.
- **Temporal Cluster:** An open-source service for durable execution of workflows as code, deployed using a customized Helm chart.
- **Temporal Worker:** A Kotlin service that executes the actual workflows and interacts with other services.
- **PDF Generator:** A separate service for generating PDFs, isolated from the accounting service to handle compute and memory-intensive tasks.
- **PostgreSQL Database:** The database for storing accounting data, managed as a service.
- **Storage:** S3 storage for storing generated PDFs and other files, managed as a service.

## Getting Started

To get started with the Accounting System, follow these steps:

1. Clone the repository:
