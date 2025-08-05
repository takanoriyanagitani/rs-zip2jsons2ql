use std::io;

use io::Read;

use std::fs::File;

use std::path::PathBuf;

use std::collections::HashMap;
use std::process::ExitCode;

use tokio::net::TcpListener;

use zip::ZipArchive;

use rs_zip2jsons2ql::async_graphql;

use async_graphql::dataloader::DataLoader;

use async_graphql::Context;
use async_graphql::Object;
use async_graphql::Request;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

use rs_zip2jsons2ql::ZipItemsLoader;
use rs_zip2jsons2ql::ZipToJsons;

pub struct ZipLoader {
    pub dir: PathBuf,
    pub limit: u64,
}

impl ZipToJsons for ZipLoader {
    type Error = &'static str;

    fn basename2jsons(
        &self,
        basename: &str,
        keys: &[String],
    ) -> Result<HashMap<String, String>, Self::Error> {
        let b: PathBuf = basename.into();
        let base: &str = b
            .file_name()
            .and_then(|o| o.to_str())
            .ok_or("invalid basename")?;

        let full: PathBuf = self.dir.join(base);
        let f: File = File::open(&full).map_err(|e| {
            eprintln!("{e}:{full:#?}");
            "unable to open the zip file"
        })?;

        let mut z: ZipArchive<_> = ZipArchive::new(f).map_err(|_| "invalid zip file")?;

        let mut h = HashMap::new();

        for key in keys {
            let zfile = z.by_name(key).map_err(|e| {
                eprintln!("{e}:{key}");
                "unable to get the item"
            })?;
            let mut taken = zfile.take(self.limit);
            let mut buf = String::new();
            taken
                .read_to_string(&mut buf)
                .map_err(|_| "invalid string")?;
            h.insert(key.into(), buf);
        }

        Ok(h)
    }
}

#[derive(Clone)]
pub struct LoaderSource {
    pub dir: PathBuf,
    pub limit: u64,
}

impl LoaderSource {
    pub fn from_env() -> Self {
        let dir = std::env::var("ZIP_DIR")
            .unwrap_or_else(|_| "./zips.d".to_string())
            .into();

        let limit: u64 = std::env::var("ITEM_SIZE_LIMIT")
            .ok()
            .and_then(|s| str::parse(s.as_str()).ok())
            .unwrap_or(1048576);

        Self { dir, limit }
    }
}

impl LoaderSource {
    pub fn into_data_loader(
        self,
        req: &Request,
    ) -> Result<DataLoader<ZipItemsLoader<ZipLoader>>, io::Error> {
        let l = ZipLoader {
            dir: self.dir,
            limit: self.limit,
        };
        rs_zip2jsons2ql::req2loader(req, l).map(|d| d.max_batch_size(3))
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn json<'a>(
        &self,
        ctx: &Context<'a>,
        item_name: String,
        basename: Option<String>,
    ) -> Result<String, io::Error> {
        let _ = basename;
        let loader = ctx
            .data::<DataLoader<ZipItemsLoader<ZipLoader>>>()
            .map_err(|_| io::Error::other("data loader missing"))?;
        loader
            .load_one(item_name)
            .await
            .map(|o| o.unwrap_or_default())
            .map_err(io::Error::other)
    }
}

type ZipJsonSchema = Schema<Query, EmptyMutation, EmptySubscription>;

async fn req2res(l: &LoaderSource, s: &ZipJsonSchema, req: GraphQLRequest) -> GraphQLResponse {
    let q: &Request = &req.0;
    let odl: Option<DataLoader<_>> = l.clone().into_data_loader(q).ok();
    let req: Request = req.into_inner();
    match odl {
        None => s.execute(req).await.into(),
        Some(dl) => {
            let req: Request = req.data(dl);
            s.execute(req).await.into()
        }
    }
}

fn env2addr_port() -> Result<String, io::Error> {
    std::env::var("LISTEN_ADDR").map_err(io::Error::other)
}

async fn sub() -> Result<(), io::Error> {
    let ls: LoaderSource = LoaderSource::from_env();
    let ap: String = env2addr_port()?;

    let sch: ZipJsonSchema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();
    let sdl: String = sch.sdl();
    std::fs::write("./zip2jsons2ql.gql", sdl.as_bytes())?;

    let lis = TcpListener::bind(ap).await?;

    let app = axum::Router::new().route(
        "/",
        axum::routing::post(|req| async move { req2res(&ls, &sch, req).await }),
    );

    axum::serve(lis, app).await
}

#[tokio::main]
async fn main() -> ExitCode {
    match sub().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("An error occurred: {err}");
            ExitCode::FAILURE
        }
    }
}
