use axum::routing::get;
use axum::{response::Html, Extension};
use reqwest::{IntoUrl, Url};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

lazy_static::lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("templates/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("{:#?}", TEMPLATES.get_template_names().collect::<Vec<_>>());

    let docker_client = DockerClient::new("http://localhost:42069");

    let app = axum::Router::new()
        .route("/infos", get(render_docker_infos))
        .route("/", get(render_root))
        .route("/containers", get(render_containers))
        .layer(Extension(docker_client));
    axum::serve(listener, app).await.unwrap();
}

async fn render_docker_infos(Extension(docker_client): Extension<DockerClient>) -> Html<String> {
    let version = docker_client.version().await;
    let context = Context::from_serialize(&version).unwrap();

    println!("{context:#?}");

    let rendered = TEMPLATES.render("infos.html", &context).unwrap();

    Html(rendered)
}

async fn render_containers(Extension(docker_client): Extension<DockerClient>) {
    let containers = docker_client.list_containers().await;
    println!("{containers:#?}");
    let context = Context::from_serialize(&containers).unwrap();
}

async fn render_root() -> Html<String> {
    let rendered = TEMPLATES.render("index.html", &Context::new()).unwrap();

    Html(rendered)
}

#[derive(Clone, Debug)]
struct DockerClient {
    transport: reqwest::Client,
    api_url: Url,
}

impl DockerClient {
    fn new(url: impl IntoUrl) -> Self {
        let api_url = url.into_url().unwrap();
        Self {
            transport: reqwest::Client::new(),
            api_url,
        }
    }

    async fn version(&self) -> Version {
        let url = format!("{}/version", self.api_url);
        self.transport
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<Version>()
            .await
            .unwrap()
    }

    async fn list_containers(&self) -> Containers {
        let url = format!("{}/containers/json", self.api_url);
        let containers = self
            .transport
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<Vec<Container>>()
            .await
            .unwrap();
        Containers { containers }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct Version {
    version: String,
    api_version: String,
    os: String,
    arch: String,
    kernel_version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct Containers {
    containers: Vec<Container>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct Container {
    id: String,
    names: Vec<String>,
    state: String,
    status: String,
}
