FROM postgres:16-bullseye

LABEL maintainer="postgres pg cron"

RUN apt-get update && apt-get -y install postgresql-16-cron

RUN mkdir -p /docker-entrypoint-initdb.d
