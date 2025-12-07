use crate::hardware::{HardwareProfile, ProfileDatabase};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, path::PathBuf, time::Duration};
use tokio::{fs, io::AsyncWriteExt};

const DEFAULT_ENDPOINT: &str = "https://profiles.keyrx.dev/community.json";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(8);
const CACHE_FILE_NAME: &str = "community_profiles.json";

/// Profile bundle served by the cloud service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProfileBundle {
    pub updated_at: DateTime<Utc>,
    pub profiles: Vec<HardwareProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedBundle {
    etag: Option<String>,
    bundle: CloudProfileBundle,
}

/// Outcome of a cloud sync attempt.
#[derive(Debug, Clone)]
pub enum CloudSyncOutcome {
    Fresh {
        database: ProfileDatabase,
        updated_at: DateTime<Utc>,
        etag: Option<String>,
    },
    Unchanged {
        database: ProfileDatabase,
        updated_at: DateTime<Utc>,
        etag: Option<String>,
    },
    Empty,
}

#[derive(Debug, Clone)]
pub struct CloudProfileSync<C = HttpProfileClient> {
    client: C,
    cache_path: PathBuf,
}

impl CloudProfileSync<HttpProfileClient> {
    /// Create a syncer backed by the HTTP client and cache under the user's cache dir.
    pub fn http_with_default_cache() -> Result<Self> {
        let cache_path = default_cache_path()?;
        Ok(Self::new(HttpProfileClient::default(), cache_path))
    }
}

impl<C> CloudProfileSync<C>
where
    C: ProfileClient + Send + Sync,
{
    pub fn new(client: C, cache_path: impl Into<PathBuf>) -> Self {
        Self {
            client,
            cache_path: cache_path.into(),
        }
    }

    /// Fetch remote profiles, write cache when changed, and return the resulting database.
    pub async fn refresh(&self) -> Result<CloudSyncOutcome> {
        let cached = self.read_cache().await?;
        let cached_etag = cached.as_ref().and_then(|c| c.etag.as_deref());
        let fetch_result = self.client.fetch_bundle(cached_etag).await?;

        match fetch_result.state {
            FetchState::NotModified => {
                if let Some(bundle) = cached {
                    let db = ProfileDatabase::from_profiles(bundle.bundle.profiles.clone());
                    return Ok(CloudSyncOutcome::Unchanged {
                        database: db,
                        updated_at: bundle.bundle.updated_at,
                        etag: bundle.etag,
                    });
                }

                Ok(CloudSyncOutcome::Empty)
            }
            FetchState::Modified(bundle) => {
                let etag = fetch_result.etag.clone();
                self.write_cache(&bundle, fetch_result.etag.as_deref())
                    .await?;
                let db = ProfileDatabase::from_profiles(bundle.profiles.clone());
                Ok(CloudSyncOutcome::Fresh {
                    database: db,
                    updated_at: bundle.updated_at,
                    etag,
                })
            }
        }
    }

    /// Load the cached profiles without performing a network request.
    pub async fn load_cached(&self) -> Result<Option<ProfileDatabase>> {
        let cached = self.read_cache().await?;
        Ok(cached.map(|c| ProfileDatabase::from_profiles(c.bundle.profiles)))
    }

    async fn read_cache(&self) -> Result<Option<CachedBundle>> {
        match fs::read(&self.cache_path).await {
            Ok(bytes) => {
                let cached: CachedBundle = serde_json::from_slice(&bytes)
                    .context("failed to parse cached cloud profiles")?;
                Ok(Some(cached))
            }
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err).context("failed to read cached cloud profiles"),
        }
    }

    async fn write_cache(&self, bundle: &CloudProfileBundle, etag: Option<&str>) -> Result<()> {
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create cache dir {parent:?}"))?;
        }

        let cached = CachedBundle {
            etag: etag.map(|e| e.to_owned()),
            bundle: bundle.clone(),
        };

        let encoded = serde_json::to_vec_pretty(&cached)
            .context("failed to serialize cached cloud profiles")?;
        let mut file = fs::File::create(&self.cache_path)
            .await
            .with_context(|| format!("failed to open cache file {:?}", self.cache_path))?;
        file.write_all(&encoded)
            .await
            .context("failed to write cache file")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FetchResult {
    state: FetchState,
    etag: Option<String>,
}

impl FetchResult {
    fn modified(bundle: CloudProfileBundle, etag: Option<String>) -> Self {
        Self {
            state: FetchState::Modified(bundle),
            etag,
        }
    }

    fn not_modified(etag: Option<String>) -> Self {
        Self {
            state: FetchState::NotModified,
            etag,
        }
    }
}

#[derive(Debug, Clone)]
enum FetchState {
    Modified(CloudProfileBundle),
    NotModified,
}

#[async_trait]
pub trait ProfileClient {
    async fn fetch_bundle(&self, etag: Option<&str>) -> Result<FetchResult>;
}

#[derive(Debug, Clone)]
pub struct HttpProfileClient {
    client: reqwest::Client,
    endpoint: String,
    timeout: Duration,
}

impl Default for HttpProfileClient {
    fn default() -> Self {
        Self::new(DEFAULT_ENDPOINT, DEFAULT_TIMEOUT)
    }
}

impl HttpProfileClient {
    pub fn new(endpoint: impl Into<String>, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("keyrx-core/1.0 (cloud-profile-sync)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            endpoint: endpoint.into(),
            timeout,
        }
    }

    pub fn with_client(client: reqwest::Client, endpoint: impl Into<String>) -> Self {
        Self {
            client,
            endpoint: endpoint.into(),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

#[async_trait]
impl ProfileClient for HttpProfileClient {
    async fn fetch_bundle(&self, etag: Option<&str>) -> Result<FetchResult> {
        let mut request = self.client.get(&self.endpoint).timeout(self.timeout);
        if let Some(tag) = etag {
            request = request.header(header::IF_NONE_MATCH, tag);
        }

        let response = request
            .send()
            .await
            .context("failed to fetch community profiles")?;

        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(FetchResult::not_modified(
                etag.map(std::string::ToString::to_string),
            ));
        }

        if !response.status().is_success() {
            return Err(anyhow!(
                "unexpected status {} when fetching community profiles",
                response.status()
            ));
        }

        let etag = response
            .headers()
            .get(header::ETAG)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_owned());

        let bundle = response
            .json::<CloudProfileBundle>()
            .await
            .context("failed to parse community profiles")?;

        Ok(FetchResult::modified(bundle, etag))
    }
}

fn default_cache_path() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow!("unable to determine cache directory for profiles"))?;
    Ok(cache_dir.join("keyrx").join(CACHE_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    #[derive(Clone, Default)]
    struct StubClient {
        responses: Arc<Mutex<Vec<FetchResult>>>,
    }

    #[async_trait]
    impl ProfileClient for StubClient {
        async fn fetch_bundle(&self, _etag: Option<&str>) -> Result<FetchResult> {
            let mut guard = self
                .responses
                .lock()
                .map_err(|_| anyhow!("stub poisoned"))?;
            guard.pop().ok_or_else(|| anyhow!("no stub responses left"))
        }
    }

    fn bundle(vendor_id: u16, product_id: u16, updated_at: DateTime<Utc>) -> CloudProfileBundle {
        CloudProfileBundle {
            updated_at,
            profiles: vec![HardwareProfile::new(
                vendor_id,
                product_id,
                "Community Board",
                crate::hardware::TimingConfig::default(),
                crate::hardware::ProfileSource::Community,
            )],
        }
    }

    fn ts(year: i32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, 1, day, 0, 0, 0)
            .single()
            .unwrap_or_else(|| Utc::now())
    }

    #[tokio::test]
    async fn refresh_fetches_and_caches_when_missing() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("profiles.json");

        let client = StubClient::default();
        client.responses.lock().unwrap().push(FetchResult::modified(
            bundle(0x1234, 0x0001, ts(2025, 2)),
            Some("etag-1".to_string()),
        ));

        let sync = CloudProfileSync::new(client, &cache_path);
        let outcome = sync.refresh().await.expect("refresh");

        match outcome {
            CloudSyncOutcome::Fresh { database, etag, .. } => {
                assert_eq!(etag.as_deref(), Some("etag-1"));
                let profile = database.lookup(0x1234, 0x0001);
                assert!(profile.is_some());
            }
            _ => panic!("expected Fresh outcome"),
        }

        let cached = fs::read(&cache_path).await.expect("cache file exists");
        assert!(!cached.is_empty());
    }

    #[tokio::test]
    async fn refresh_returns_unchanged_when_not_modified() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("profiles.json");
        let cached_bundle = CachedBundle {
            etag: Some("etag-1".to_string()),
            bundle: bundle(0x2222, 0x1111, ts(2025, 3)),
        };
        let encoded = serde_json::to_vec(&cached_bundle).expect("serialize");
        fs::write(&cache_path, encoded).await.expect("write cache");

        let client = StubClient::default();
        client
            .responses
            .lock()
            .unwrap()
            .push(FetchResult::not_modified(Some("etag-1".to_string())));

        let sync = CloudProfileSync::new(client, &cache_path);
        let outcome = sync.refresh().await.expect("refresh");

        match outcome {
            CloudSyncOutcome::Unchanged { database, .. } => {
                let profile = database.lookup(0x2222, 0x1111);
                assert!(profile.is_some());
            }
            other => panic!("unexpected outcome: {:?}", other),
        }
    }

    #[tokio::test]
    async fn refresh_updates_when_remote_changes() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("profiles.json");
        let cached_bundle = CachedBundle {
            etag: Some("etag-1".to_string()),
            bundle: bundle(0x2222, 0x1111, ts(2025, 1)),
        };
        let encoded = serde_json::to_vec(&cached_bundle).expect("serialize");
        fs::write(&cache_path, encoded).await.expect("write cache");

        let client = StubClient::default();
        client.responses.lock().unwrap().push(FetchResult::modified(
            bundle(0x3333, 0x0002, ts(2025, 4)),
            Some("etag-2".to_string()),
        ));

        let sync = CloudProfileSync::new(client, &cache_path);
        let outcome = sync.refresh().await.expect("refresh");

        match outcome {
            CloudSyncOutcome::Fresh { database, etag, .. } => {
                assert_eq!(etag.as_deref(), Some("etag-2"));
                assert!(database.lookup(0x3333, 0x0002).is_some());
            }
            other => panic!("unexpected outcome: {:?}", other),
        }
    }
}
