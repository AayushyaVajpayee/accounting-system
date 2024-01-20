# Use Ubuntu as the base image
FROM ubuntu:latest

# Set environment variables
ENV AWS_CLI_VERSION="2.15.12"
ENV KUBECTL_VERSION="1.29.1"

# Update and install necessary packages
RUN apt-get update -y && \
    apt-get install -y \
        curl \
        unzip \
        && \
    rm -rf /var/lib/apt/lists/*

# Install AWS CLI v2
RUN curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64-${AWS_CLI_VERSION}.zip" -o "awscliv2.zip" && \
    unzip awscliv2.zip && \
     ./aws/install && \
    rm -rf awscliv2.zip ./aws

# Install kubectl
RUN curl -LO "https://dl.k8s.io/release/v${KUBECTL_VERSION}/bin/linux/amd64/kubectl" && \
    install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl && \
    rm kubectl

# Set the working directory
WORKDIR /app

# Default command to run when the container starts
CMD ["bash"]
