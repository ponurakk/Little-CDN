use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
    ),
    components(
    ),
    tags(
        (name = "Little CDN", description = "Little CDN documentation")
    )
)]
pub struct ApiDoc;
