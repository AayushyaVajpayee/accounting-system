# syntax=docker/dockerfile:1
FROM rust:1.73 as builder
WORKDIR /usr/src/accounting-system-workspace
COPY . .
RUN apt-get update && apt-get install -y musl-tools
RUN  --mount=type=cache,target=/usr/local/cargo/registry \
    target=/usr/src/accounting-system-workspace/target \
    rustup target add x86_64-unknown-linux-musl
RUN  --mount=type=cache,target=/usr/local/cargo/registry \
     --mount=type=cache,target=/usr/src/accounting-system-workspace/target \
     cargo install --target x86_64-unknown-linux-musl --path ./accounting_system/

FROM amazonlinux:2023.2.20231026.0 as runtime
#FROM public.ecr.aws/amazonlinux/amazonlinux:2023-minimal as runtime
#RUN #apt-get update && apt-get install -y apt-get install camusl-tools && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/accounting-system /usr/local/bin/accounting-system
EXPOSE 8080
CMD ["accounting-system"]