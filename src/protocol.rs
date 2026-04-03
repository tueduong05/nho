use std::io::Cursor;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::{
    io::{self as TokioIo, AsyncReadExt, AsyncWriteExt},
    net::TcpStream as TokioTcpStream,
};

use crate::command::Command;

pub enum Response {
    Ok,
    Nil,
    Data(Bytes),
    Error(String),
}

impl Response {
    fn serialize(self, buffer: &mut BytesMut) {
        match self {
            Response::Ok => buffer.put_slice(b"+OK\r\n"),
            Response::Nil => buffer.put_slice(b"$-1\r\n"),
            Response::Data(data) => {
                buffer.put_u8(b'$');
                buffer.put_slice(data.len().to_string().as_bytes());
                buffer.put_slice(b"\r\n");
                buffer.put_slice(&data);
                buffer.put_slice(b"\r\n");
            }
            Response::Error(message) => {
                buffer.put_slice(b"-ERR ");
                buffer.put_slice(message.as_bytes());
                buffer.put_slice(b"\r\n");
            }
        }
    }
}

pub struct Connection {
    stream: TokioTcpStream,
    buffer: BytesMut,
    write_buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TokioTcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(65_536),
            write_buffer: BytesMut::with_capacity(65_536),
        }
    }

    pub async fn read_frame(&mut self) -> TokioIo::Result<Option<Command>> {
        loop {
            if let Some(command) = self.parse_frame()? {
                return Ok(Some(command));
            }

            if !self.write_buffer.is_empty() {
                self.flush().await?;
            }

            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err(TokioIo::Error::new(
                        TokioIo::ErrorKind::ConnectionReset,
                        "Connection reset by peer mid-frame",
                    ))
                };
            }
        }
    }

    pub fn write_response(&mut self, response: Response) {
        response.serialize(&mut self.write_buffer);
    }

    pub async fn flush(&mut self) -> TokioIo::Result<()> {
        if !self.write_buffer.is_empty() {
            self.stream.write_all(&self.write_buffer).await?;
            self.write_buffer.clear();
        }
        self.stream.flush().await
    }

    fn parse_frame(&mut self) -> TokioIo::Result<Option<Command>> {
        let mut cursor = Cursor::new(&self.buffer[..]);

        if !cursor.has_remaining() {
            return Ok(None);
        }

        if cursor.get_u8() != b'*' {
            return Err(TokioIo::Error::new(
                TokioIo::ErrorKind::InvalidData,
                "Expected array prefix '*'",
            ));
        }

        let num_elements = match self.read_line(&mut cursor)? {
            Some(line) => self.parse_number(&line)?,
            None => return Ok(None),
        };

        let mut args = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            if !cursor.has_remaining() {
                return Ok(None);
            }

            if cursor.get_u8() != b'$' {
                return Err(TokioIo::Error::new(
                    TokioIo::ErrorKind::InvalidData,
                    "Expected bulk string prefix '$'",
                ));
            }

            let data_len = match self.read_line(&mut cursor)? {
                Some(line) => self.parse_number(&line)?,
                None => return Ok(None),
            };

            let start = cursor.position() as usize;
            let end = start + data_len;

            if cursor.get_ref().len() < end + 2 {
                return Ok(None);
            }

            if &cursor.get_ref()[end..end + 2] != b"\r\n" {
                return Err(TokioIo::Error::new(
                    TokioIo::ErrorKind::InvalidData,
                    "Missing CRLF terminator",
                ));
            }

            let data = self.buffer.clone().freeze().slice(start..end);

            cursor.set_position((end + 2) as u64);
            args.push(data);
        }

        let consumed = cursor.position() as usize;
        self.buffer.advance(consumed);

        Ok(Some(Command::from_args(args)))
    }

    fn read_line(&self, cursor: &mut Cursor<&[u8]>) -> TokioIo::Result<Option<Vec<u8>>> {
        let start = cursor.position() as usize;
        let bytes = &cursor.get_ref()[start..];

        if let Some(i) = bytes.iter().position(|&b| b == b'\n') {
            if i > 0 && bytes[i - 1] == b'\r' {
                let line = bytes[..i - 1].to_vec();
                cursor.set_position((start + i + 1) as u64);

                return Ok(Some(line));
            } else {
                return Err(TokioIo::Error::new(
                    TokioIo::ErrorKind::InvalidData,
                    "Invalid line terminator (lone LF)",
                ));
            }
        }

        Ok(None)
    }

    fn parse_number(&self, bytes: &[u8]) -> TokioIo::Result<usize> {
        str::from_utf8(bytes)
            .map_err(|_| {
                TokioIo::Error::new(TokioIo::ErrorKind::InvalidData, "Invalid UTF-8 sequence")
            })?
            .parse::<usize>()
            .map_err(|_| {
                TokioIo::Error::new(TokioIo::ErrorKind::InvalidData, "Invalid number format")
            })
    }
}
