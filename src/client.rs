

use std::path::{Path, PathBuf};

use async_std::net::TcpStream;
//use async_std::prelude::*;

//use bincode::config::standard;
use bincode::{Decode, Encode};

//use crate::filesystem::FileSystemManager;
use crate::encryption::{EncryptionManager, KeyPair};
use crate::async_io::AsyncConnection;
use crate::protocol::{NfsMessage, NfsOperation, NfsResponse, FileStat, DirEntry};
use crate::error::NfsError;

pub struct NfsClient {
    server_address: String,
    encryption_manager: EncryptionManager,
    connection: Option<AsyncConnection>,
    mount_point: PathBuf,
}

impl NfsClient {
    pub fn new(server_address: String, mount_point: PathBuf, keypair: KeyPair) -> Self {
        Self {
            server_address,
            encryption_manager: EncryptionManager::new(keypair),
            connection: None,
            mount_point,
        }
    }
    
    pub async fn connect(&mut self) -> Result<(), NfsError> {
        log::info!("Connecting to NFS server at {}", self.server_address);
        
        let stream = TcpStream::connect(&self.server_address).await?;
        let mut connection = AsyncConnection::new(stream);
        
        // Perform handshake and setup encryption
        self.perform_handshake(&mut connection).await?;
        
        self.connection = Some(connection);
        log::info!("Connected to NFS server");
        
        Ok(())
    }
    
    async fn perform_handshake(&self, connection: &mut AsyncConnection) -> Result<(), NfsError> {
        // Exchange public keys and establish secure channel
        // ... implementation details ...
        Ok(())
    }
    
    pub async fn disconnect(&mut self) -> Result<(), NfsError> {
        self.connection = None;
        Ok(())
    }
    
    async fn send_operation(&mut self, operation: NfsOperation) -> Result<Vec<u8>, NfsError> {
        let connection = self.connection.as_mut()
            .ok_or_else(|| NfsError::NotConnected)?;
            
        let message = NfsMessage { operation };
        connection.send_message(&message).await?;
        
        match connection.receive_message::<NfsResponse>().await? {
            NfsResponse::Success(data) => Ok(data),
            NfsResponse::Error(msg) => Err(NfsError::RemoteError(msg)),
        }
    }
    
    pub async fn read_file(&mut self, path: impl AsRef<Path>, offset: u64, length: u64) -> Result<Vec<u8>, NfsError> {
        let operation = NfsOperation::Read {
            path: path.as_ref().to_path_buf(),
            offset,
            length,
        };
        
        self.send_operation(operation).await
    }
    
    pub async fn write_file(&mut self, path: impl AsRef<Path>, offset: u64, data: Vec<u8>) -> Result<(), NfsError> {
        let operation = NfsOperation::Write {
            path: path.as_ref().to_path_buf(),
            offset,
            data,
        };
        
        let _ = self.send_operation(operation).await?;
        Ok(())
    }
    
    pub async fn create_file(&mut self, path: impl AsRef<Path>, mode: u32) -> Result<(), NfsError> {
        let operation = NfsOperation::Create {
            path: path.as_ref().to_path_buf(),
            mode,
        };
        
        let _ = self.send_operation(operation).await?;
        Ok(())
    }
    
    pub async fn create_directory(&mut self, path: impl AsRef<Path>, mode: u32) -> Result<(), NfsError> {
        let operation = NfsOperation::Mkdir {
            path: path.as_ref().to_path_buf(),
            mode,
        };
        
        let _ = self.send_operation(operation).await?;
        Ok(())
    }
    
    pub async fn remove(&mut self, path: impl AsRef<Path>) -> Result<(), NfsError> {
        let operation = NfsOperation::Remove {
            path: path.as_ref().to_path_buf(),
        };
        
        let _ = self.send_operation(operation).await?;
        Ok(())
    }
    
    /// An asynchronous function designed to retrieve metadata (stat information) about a file or directory specified by its path. 
    /// It interacts with the network file system to perform this operation. The function returns a `Result` type that, on success, 
    /// contains a `FileStat` structure with the file's metadata, or an `NfsError` if something goes wrong.
    pub async fn stat(&mut self, path: impl AsRef<Path>) -> Result<FileStat, NfsError> {
        let operation = NfsOperation::Stat {
            path: path.as_ref().to_path_buf(),
        };
        
        let data = self.send_operation(operation).await?;
        // let stat: FileStat = bincode::decode_from_slice(&data, bincode::config::standard())?;
        // Ok(stat)
        match bincode::decode_from_slice(&data, bincode::config::standard()) {
            Ok((stat, _)) => Ok(stat),
            Err(e) => Err(NfsError::DeserializationError(e.to_string()))
        }
    }
    
    pub async fn read_dir(&mut self, path: impl AsRef<Path>) -> Result<Vec<DirEntry>, NfsError> {
        let operation = NfsOperation::Readdir {
            path: path.as_ref().to_path_buf(),
        };
        
        let data = self.send_operation(operation).await?;
        let (entries, _) = bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(entries)
    }
}