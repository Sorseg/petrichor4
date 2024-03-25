use std::io::Cursor;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext},
    prelude::{Asset, TypePath},
};
use obj::{load_obj, Obj, Position};
use thiserror::Error;

pub(crate) struct Collider {
    name: String,
    pos: Vec<Position>,
    indices: Vec<u16>,
}

/// Represents a server-side view of a level
/// contains e.g.
/// - colliders
/// - triggers
/// - where the player spawns
#[derive(TypePath, Asset)]
pub(crate) struct PetriObj {
    // TODO
    pub(crate) colliders: Vec<Collider>,
}

#[derive(Default)]
pub(crate) struct PetriObjLoader;

#[derive(Debug, Error)]
pub(crate) enum PetriObjLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cannot parse obj file: {0}")]
    Obj(#[from] obj::ObjError),
}

impl AssetLoader for PetriObjLoader {
    type Asset = PetriObj;
    type Settings = ();

    // make this a real error
    type Error = PetriObjLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let mut cursor = Cursor::new(&bytes);
            let mut colliders = vec![];
            while cursor.position() < bytes.len() as u64 {
                let obj: Obj<Position> = load_obj(&mut cursor)?;
                let Some(name) = obj.name else { continue };
                if let Some(name) = name.strip_prefix("collider_") {
                    colliders.push(Collider {
                        name: name.to_string(),
                        pos: obj.vertices,
                        indices: obj.indices,
                    })
                }
            }
            Ok(PetriObj { colliders })
        })
    }
}
