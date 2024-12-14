use libp2p::{
    request_response::{ProtocolName, RequestResponseCodec},
    bytes::Bytes,
};
use async_trait::async_trait;
use std::{io, str};

#[derive(Clone)]
pub struct FileTransferProtocol();

impl ProtocolName for FileTransferProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/file-transfer/1.0.0"
    }
}

#[derive(Debug, Clone)]
pub struct FileRequest {
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct FileResponse {
    pub content: Vec<u8>,
}

#[derive(Clone)]
pub struct FileTransferCodec;

#[async_trait]
impl RequestResponseCodec for FileTransferCodec {
    type Protocol = FileTransferProtocol;
    type Request = FileRequest;
    type Response = FileResponse;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let data = read_length_prefixed(io, 1024).await?;
        let filename = str::from_utf8(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .to_string();
        Ok(FileRequest { filename })
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let data = read_length_prefixed(io, 16 * 1024).await?;
        Ok(FileResponse { content: data })
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        FileRequest { filename }: FileRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, filename.as_bytes()).await?;
        io.close().await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        FileResponse { content }: FileResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, &content).await?;
        io.close().await
    }
}