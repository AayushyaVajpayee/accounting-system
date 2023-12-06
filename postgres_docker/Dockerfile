# Use the official PostgreSQL image as the base image
FROM postgres:latest

# Install necessary packages
RUN apt-get update \
    && apt-get -y install postgresql-16-cron

# Copy the pg_cron extension SQL script
COPY pg_cron.sql /docker-entrypoint-initdb.d/pg_cron.sql
COPY config.file /etc/postgresql/conf.d/config.file
RUN echo "shared_preload_libraries = 'pg_cron'" >> /usr/share/postgresql/postgresql.conf.sample
RUN echo "cron.database_name = 'postgres'" >> /usr/share/postgresql/postgresql.conf.sample