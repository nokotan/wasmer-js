use virtual_fs::FileOpener;

use crate::codefs::client::CodeFSClient;
use crate::codefs::file::CodeFSVirtualFile;
use crate::vscode::stat::FileStat;
use virtual_fs::FileSystem;

impl FileOpener for CodeFSClient {
    fn open(
        &self,
        path: &std::path::Path,
        conf: &virtual_fs::OpenOptionsConfig,
    ) -> virtual_fs::Result<Box<dyn virtual_fs::VirtualFile + Send + Sync + 'static>> {
        let buffer = if conf.read() {
            self.read_all(path)?
        } else {
            Vec::new()
        };

        let metadata = if conf.read() {
            self.metadata(path).unwrap()
        } else {
            FileStat::new().into()
        };

        Ok(Box::new(CodeFSVirtualFile::new(
            self.clone(),
            metadata,
            path.to_owned(),
            buffer,
        )))
    }
}
