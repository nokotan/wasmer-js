use futures::future::BoxFuture;
use std::path::Path;
use tokio::sync::mpsc::{self, Sender};
use virtual_fs::{FsError, Metadata, ReadDir};
use virtual_mio::waker::InlineWaker;

use crate::codefs::FileCommands;

#[derive(Clone, Debug)]
pub struct CodeFSClient {
    tx: Sender<FileCommands>,
}

impl CodeFSClient {
    pub fn new(tx: Sender<FileCommands>) -> Self {
        Self { tx }
    }

    pub async fn read_all(&self, path: &std::path::Path) -> Result<Vec<u8>, FsError> {
        let (tx, mut rx) = mpsc::channel(1);

        self.tx
            .send(FileCommands::ReadFile {
                path: path.into(),
                tx,
            })
            .await
            .map_err(|_| FsError::IOError)?;
        Ok(rx.recv().await.unwrap())
    }

    pub async fn write_all(&self, path: &std::path::Path, data: &[u8]) -> Result<(), FsError> {
        let (tx, mut rx) = mpsc::channel(1);

        self.tx
            .send(FileCommands::WriteFile {
                path: path.into(),
                data: data.to_vec(),
                tx,
            })
            .await
            .map_err(|_| FsError::IOError)?;
        rx.recv().await;
        Ok(())
    }
}

impl virtual_fs::FileSystem for CodeFSClient {
    fn metadata(&self, path: &std::path::Path) -> virtual_fs::Result<Metadata> {
        InlineWaker::block_on(async move {
            let (tx, mut rx) = mpsc::channel(1);

            self.tx
                .send(FileCommands::Stat {
                    path: path.into(),
                    tx,
                })
                .await
                .map_err(|_| FsError::IOError)?;
            rx.recv().await.unwrap()
        })
    }

    fn create_dir(&self, path: &std::path::Path) -> virtual_fs::Result<()> {
        InlineWaker::block_on(async move {
            let (tx, mut rx) = mpsc::channel(1);

            self.tx
                .send(FileCommands::CreateDirectory {
                    path: path.into(),
                    tx,
                })
                .await
                .map_err(|_| FsError::IOError)?;
            rx.recv().await;
            Ok(())
        })
    }

    fn read_dir(&self, path: &std::path::Path) -> virtual_fs::Result<ReadDir> {
        InlineWaker::block_on(async move {
            let (tx, mut rx) = mpsc::channel(1);

            self.tx
                .send(FileCommands::ReadDirectory {
                    path: path.into(),
                    tx,
                })
                .await
                .map_err(|_| FsError::IOError)?;
            rx.recv().await.unwrap()
        })
    }

    fn rename<'a>(&'a self, from: &'a Path, to: &'a Path) -> BoxFuture<'a, virtual_fs::Result<()>> {
        let (tx, mut rx) = mpsc::channel(1);

        Box::pin(async move {
            self.tx
                .send(FileCommands::Rename {
                    old_path: from.into(),
                    new_path: to.into(),
                    tx,
                })
                .await
                .map_err(|_| FsError::IOError)?;
            rx.recv().await;
            Ok(())
        })
    }

    fn remove_file(&self, path: &std::path::Path) -> virtual_fs::Result<()> {
        InlineWaker::block_on(async move {
            let (tx, mut rx) = mpsc::channel(1);

            self.tx
                .send(FileCommands::Delete {
                    path: path.into(),
                    tx,
                })
                .await
                .map_err(|_| FsError::IOError)?;
            rx.recv().await;
            Ok(())
        })
    }

    fn remove_dir(&self, path: &std::path::Path) -> virtual_fs::Result<()> {
        InlineWaker::block_on(async move {
            let (tx, mut rx) = mpsc::channel(1);

            self.tx
                .send(FileCommands::Delete {
                    path: path.into(),
                    tx,
                })
                .await
                .map_err(|_| FsError::IOError)?;
            rx.recv().await;
            Ok(())
        })
    }

    fn new_open_options(&self) -> virtual_fs::OpenOptions {
        virtual_fs::OpenOptions::new(self)
    }
}
