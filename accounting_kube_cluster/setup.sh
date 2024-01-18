export POSTGRES_USER_PASSWORD="dummy_pwd"
export POSTGRES_USER="dummy"
export POSTGRES_PASSWORD="postgres_pwd"
export ACCOUNTING_POSTGRES_DB="accounting_system"
export TEMPORAL_DB="temporal"
export TEMPORAL_VISIBILITY_DB="temporal_visibility"
export POSTGRES_PORT="5432"
export db_name_prefix_helm="postgres-db"
export POSTGRES_HOST="$db_name_prefix_helm-postgresql.default.svc.cluster.local"

export tem="create database $ACCOUNTING_POSTGRES_DB;
create database $TEMPORAL_DB;
create database $TEMPORAL_VISIBILITY_DB;"

yq eval '.data += {"init.sql": strenv(tem)}' -i  ./accounting_system_topology/postgres/postgres_init_configmap.yaml

kubectl apply  -f ./accounting_system_topology/postgres/postgres_init_configmap.yaml

helm install $db_name_prefix_helm oci://registry-1.docker.io/bitnamicharts/postgresql \
--set global.postgresql.auth.postgresPassword="$POSTGRES_PASSWORD" \
--set global.postgresql.auth.username="$POSTGRES_USER" \
--set global.postgresql.auth.password="$POSTGRES_USER_PASSWORD" \
--set primary.initdb.scriptsConfigMap=postgres-init-config

yq -i '.data.POSTGRES_USER=strenv(POSTGRES_USER) |
       .data.POSTGRES_PASSWORD=strenv(POSTGRES_USER_PASSWORD) |
       .data.POSTGRES_APP_DB=strenv(ACCOUNTING_POSTGRES_DB) |
       .data.POSTGRES_PORT=strenv(POSTGRES_PORT) |
       .data.POSTGRES_HOST=strenv(POSTGRES_HOST) ' ./accounting_system_topology/accounting_system/postgres_db_config_map.yaml

enc_val=$(echo -n "$POSTGRES_USER_PASSWORD" | base64) yq -i '.data.POSTGRES_PASSWORD=strenv(enc_val)' ./accounting_system_topology/accounting_system/postgres_db_pwd.yaml


helm install accounting ./temporal_helm_chart/ --values ./temporal_helm_chart/values.my_custom.postgres.yaml \
 --set server.config.persistence.default.sql.host="$POSTGRES_HOST" \
 --set server.config.persistence.default.sql.port="$POSTGRES_PORT" \
 --set server.config.persistence.default.sql.database="$TEMPORAL_DB" \
 --set server.config.persistence.default.sql.user="$POSTGRES_USER" \
 --set server.config.persistence.default.sql.password="$POSTGRES_USER_PASSWORD" \
 --set server.config.persistence.visibility.sql.host="$POSTGRES_HOST" \
 --set server.config.persistence.visibility.sql.port="$POSTGRES_PORT" \
 --set server.config.persistence.visibility.sql.database="$TEMPORAL_VISIBILITY_DB" \
 --set server.config.persistence.visibility.sql.user="$POSTGRES_USER" \
 --set server.config.persistence.visibility.sql.password="$POSTGRES_USER_PASSWORD"
