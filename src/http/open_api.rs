use utoipa::OpenApi;
use crate::model::body_kind::BodyKind;
use crate::model::echo_response::EchoResponse;

#[derive(OpenApi, Debug, Clone)]
#[openapi(
    paths(
        crate::http::ascii::ascii_handler,
        crate::http::handler::echo,
        crate::http::upload::upload,
        crate::http::cleaner::cleaner
    ),
    components(schemas(BodyKind, EchoResponse)),
    tags((name = "Echo", description = "Echo Service"))
)]
pub struct ApiDoc;