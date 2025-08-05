use std::io;

use std::path::PathBuf;

use std::collections::HashMap;

pub use async_graphql;

use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::Loader;

use async_graphql::Request;
use async_graphql::Value;
use async_graphql::Variables;

pub trait ZipToJsons: Send + Sync + 'static {
    type Error: Send + Clone + 'static;

    fn basename2jsons(
        &self,
        basename: &str,
        keys: &[String],
    ) -> Result<HashMap<String, String>, Self::Error>;
}

pub struct ZipItemsLoader<L> {
    pub basename: PathBuf,
    pub loader: L,
}

impl<L> ZipItemsLoader<L> {
    pub fn vars2str(v: &Variables, key: &str) -> Option<String> {
        match v.get(key) {
            Some(Value::String(s)) => Some(s.into()),
            _ => None,
        }
    }

    pub fn from_vars(v: &Variables, loader: L) -> Result<Self, io::Error> {
        let basename: String =
            Self::vars2str(v, "basename").ok_or(io::Error::other("basename missing"))?;
        Ok(Self {
            basename: basename.into(),
            loader,
        })
    }
}

#[async_trait::async_trait]
impl<L> Loader<String> for ZipItemsLoader<L>
where
    L: ZipToJsons,
{
    type Value = String;
    type Error = L::Error;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        eprintln!("key size: {}", keys.len());
        eprintln!("keys: {keys:#?}");
        let base: &str = self
            .basename
            .file_name()
            .and_then(|o| o.to_str())
            .unwrap_or_default();
        self.loader.basename2jsons(base, keys)
    }
}

pub struct LoaderSource {
    pub dir: PathBuf,
    pub item_limit: u64,
}

pub fn vars2loader<L>(
    vars: &Variables,
    loader: L,
) -> Result<DataLoader<ZipItemsLoader<L>>, io::Error>
where
    L: ZipToJsons,
{
    Ok(DataLoader::new(
        ZipItemsLoader::from_vars(vars, loader)?,
        tokio::spawn,
    ))
}

pub fn req2loader<L>(req: &Request, loader: L) -> Result<DataLoader<ZipItemsLoader<L>>, io::Error>
where
    L: ZipToJsons,
{
    vars2loader(&req.variables, loader)
}
