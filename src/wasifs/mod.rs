use std::sync::Arc;
use virtual_fs::{FileOpener, FileSystem, TmpFileSystem, UnionFileSystem};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::codefs::fs::CodeFS;
use crate::codefs::vscode::uri::Uri;

type RwLock<T> = tokio::sync::RwLock<T>;

#[derive(Debug, wasm_bindgen_derive::TryFromJsValue)]
#[wasm_bindgen]
pub struct WasiFS {
    pub(crate) fs: Arc<RwLock<UnionFileSystem>>,
}

#[wasm_bindgen]
impl WasiFS {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let union_fs = UnionFileSystem::new();
        let fs = Arc::new(RwLock::new(union_fs));
        let root_fs = TmpFileSystem::new();
        root_fs.create_dir(&std::path::PathBuf::from("/workspace"));

        {
            let mut union_fs = fs.try_write().expect("cannot write");
            union_fs.mount("root", "/", false, Box::new(root_fs), None);
        }

        Self { fs }
    }

    pub fn mount(&mut self, base_uri: Uri, mount_point: String) {
        let mut vscode_fs = CodeFS::new(base_uri);
        let mut fs = self.fs.clone().try_write_owned().expect("cannot write");

        fs.create_dir(mount_point.as_ref());
        fs.mount(
            "vscode",
            &mount_point,
            false,
            Box::new(vscode_fs.create_client()),
            None,
        );

        spawn_local(async move { vscode_fs.poll().await });
    }

    pub fn unmount(&mut self, mount_point: String) {
        let mut fs = self.fs.clone().try_write_owned().expect("cannot write");
        fs.unmount(&mount_point);
        fs.remove_dir(mount_point.as_ref());
    }

    pub fn clone(&self) -> Self {
        Self {
            fs: self.fs.clone()
        }
    }
}

impl FileSystem for WasiFS {
    fn read_dir(&self, path: &std::path::Path) -> virtual_fs::Result<virtual_fs::ReadDir> {
        let fs = self.fs.clone().try_read_owned().expect("cannot read");
        fs.read_dir(path)
    }

    fn create_dir(&self, path: &std::path::Path) -> virtual_fs::Result<()> {
        let fs =
            self.fs.clone().try_read_owned().expect("cannot read");
        fs.create_dir(path)
    }

    fn remove_dir(&self, path: &std::path::Path) -> virtual_fs::Result<()> {
        let fs =
            self.fs.clone().try_read_owned().expect("cannot read");
        fs.remove_dir(path)
    }

    fn rename<'a>(
        &'a self,
        from: &'a std::path::Path,
        to: &'a std::path::Path,
    ) -> futures::prelude::future::BoxFuture<'a, virtual_fs::Result<()>> {
        let arcfs = self.fs.clone();
        Box::pin(async move {
            arcfs.try_read_owned().expect("cannot read").rename(from, to).await
        })
    }

    fn metadata(&self, path: &std::path::Path) -> virtual_fs::Result<virtual_fs::Metadata> {
        let fs =
            self.fs.clone().try_read_owned().expect("cannot read");
        fs.metadata(path)
    }

    fn remove_file(&self, path: &std::path::Path) -> virtual_fs::Result<()> {
        let fs =
            self.fs.clone().try_read_owned().expect("cannot read");
        fs.remove_file(path)
    }

    fn new_open_options(&self) -> virtual_fs::OpenOptions {
        virtual_fs::OpenOptions::new(self)
    }
}

impl FileOpener for WasiFS {
    fn open(
        &self,
        path: &std::path::Path,
        conf: &virtual_fs::OpenOptionsConfig,
    ) -> virtual_fs::Result<Box<dyn virtual_fs::VirtualFile + Send + Sync + 'static>> {
        let fs =
            self.fs.try_read().expect("cannot read");
        fs.open(path, conf)
    }
}
