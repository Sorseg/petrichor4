use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext},
    prelude::*,
};
use thiserror::Error;

pub(crate) struct BlobLoaderPlugin;

#[derive(Asset, TypePath, Debug)]
pub(crate) struct Blob(pub Vec<u8>);

impl Plugin for BlobLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<BlobAssetLoader>()
            .init_asset::<Blob>();
    }
}

#[derive(Default)]
struct BlobAssetLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BlobAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for BlobAssetLoader {
    type Asset = Blob;
    type Settings = ();
    type Error = BlobAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            info!("Loading Blob...");
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            Ok(Blob(bytes))
        })
    }
}
