use bytes::Bytes;
use http::Method;
use crate::codec::{Encoding, FromReq, FromRes, IntoReq, IntoRes};
use crate::request::{ClientReq, Req};
use crate::response::{ClientRes, Res};
use crate::ServerFnError;

pub struct Bitcode;

impl Encoding for Bitcode {
    const CONTENT_TYPE: &'static str = "application/bitcode";
    const METHOD: Method = Method::POST;
}

impl<CustErr, T, Request> IntoReq<Bitcode, Request, CustErr> for T
where
    Request: ClientReq<CustErr>,
    T: bitcode::Encode + Send,
{
    fn into_req(
        self,
        path: &str,
        accepts: &str,
    ) -> Result<Request, ServerFnError<CustErr>> {
        Request::try_new_post_bytes(
            path,
            accepts,
            Bitcode::CONTENT_TYPE,
            Bytes::from(bitcode::encode(&self)),
        )
    }
}

impl<CustErr, T, Request> FromReq<Bitcode, Request, CustErr> for T
where
    Request: Req<CustErr> + Send + 'static,
    T: bitcode::Decode,
{
    async fn from_req(req: Request) -> Result<Self, ServerFnError<CustErr>> {
        let body_bytes = req.try_into_bytes().await?;
        bitcode::decode(body_bytes.as_ref())
            .map_err(|e| ServerFnError::Args(e.to_string()))
    }
}

impl<CustErr, T, Response> IntoRes<Bitcode, Response, CustErr> for T
where
    Response: Res<CustErr>,
    T: bitcode::Encode + Send,
{
    async fn into_res(self) -> Result<Response, ServerFnError<CustErr>> {
        let mut buffer: Vec<u8> = Vec::new();
        ciborium::ser::into_writer(&self, &mut buffer)
            .map_err(|e| ServerFnError::Serialization(e.to_string()))?;
        Response::try_from_bytes(Bitcode::CONTENT_TYPE, Bytes::from(bitcode::encode(&self)))
    }
}

impl<CustErr, T, Response> FromRes<Bitcode, Response, CustErr> for T
where
    Response: ClientRes<CustErr> + Send,
    T: bitcode::Decode + Send,
{
    async fn from_res(res: Response) -> Result<Self, ServerFnError<CustErr>> {
        let data = res.try_into_bytes().await?;
        bitcode::decode(data.as_ref())
            .map_err(|e| ServerFnError::Args(e.to_string()))
    }
}