use std::future::Future;
use std::sync::Arc;
use moka::future::Cache;
use uuid::Uuid;

pub async fn get_or_fetch_entity<T, Erro, Fut>(tenant_id: Uuid, entity_id: Uuid, cache: &Cache<(Uuid, Uuid),
    Arc<T>>, fetch: Fut)
                                           -> Result<Option<Arc<T>>, Erro>
    where
        T: Send + Sync + 'static,
        Fut: Future<Output=Result<Option<T>, Erro>>,
{
    let key = (tenant_id, entity_id);
    if let Some(entity) = cache.get(&key).await {
        return Ok(Some(entity));
    }
    let kk = fetch.await?;
    if let Some(en) = kk {
        let k = Arc::new(en);
        cache.insert(key, k.clone()).await;
        Ok(Some(k))
    } else {
        Ok(None)
    }
}



#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use async_trait::async_trait;
    use mockall::automock;
    use moka::future::Cache;
    use uuid::Uuid;
    use crate::common_utils::cache_utils::get_or_fetch_entity;

    #[derive(Debug, Clone)]
    struct E {}

    #[cfg_attr(test, automock)]
    #[async_trait]
    trait S {
        async fn fetch(&self) -> anyhow::Result<Option<E>>;
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let mut mock: MockS = MockS::new();
        mock.expect_fetch()
            .never();
        let tenant_id = Uuid::now_v7();
        let id = Uuid::now_v7();
        let cache: Cache<(Uuid, Uuid), Arc<E>> = Cache::new(1);
        cache.insert((tenant_id, id), Arc::new(E {})).await;
        let p =
            get_or_fetch_entity(tenant_id, id, &cache,
                                async {
                                    mock.fetch().await
                                }).await;
    }
    #[tokio::test]
    async fn test_cache_miss(){
        let mut mock: MockS = MockS::new();

        let tenant_id = Uuid::now_v7();
        let id = Uuid::now_v7();
        let cache: Cache<(Uuid, Uuid), Arc<E>> = Cache::new(1);
        let e =E{};
        mock.expect_fetch()
            .once()
            .return_once(||Ok(Some(e)));
        assert!(cache.get(&(tenant_id,id)).await.is_none());
        let p =
            get_or_fetch_entity(tenant_id, id, &cache,
                                async {
                                    mock.fetch().await
                                }).await;
        assert!(cache.get(&(tenant_id,id)).await.is_some())

    }
}