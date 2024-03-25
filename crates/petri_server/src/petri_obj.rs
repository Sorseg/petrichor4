use std::io::Cursor;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext},
    log::info,
    prelude::{Asset, TypePath},
};
use thiserror::Error;
use wavefront_obj::obj::{Primitive, Vertex};

pub(crate) struct Collider {
    name: String,
    pub(crate) pos: Vec<Vertex>,
    pub(crate) indices: Vec<u16>,
}

/// Represents a server-side view of a level
/// contains e.g.
/// - colliders
/// - triggers
/// - where the player spawns
#[derive(TypePath, Asset)]
pub(crate) struct PetriObj {
    pub(crate) colliders: Vec<Collider>,
}

#[derive(Default)]
pub(crate) struct PetriObjLoader;

#[derive(Debug, Error)]
pub(crate) enum PetriObjLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cannot parse obj file: {0}")]
    Obj(#[from] wavefront_obj::mtl::ParseError),
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
            // let mut cursor = Cursor::new(&bytes);

            let obj = wavefront_obj::obj::parse(String::from_utf8_lossy(&bytes))?;
            Ok(PetriObj {
                colliders: obj
                    .objects
                    .iter()
                    .filter(|o| o.name.to_ascii_lowercase().starts_with("collider_"))
                    .map(|o| Collider {
                        name: o.name.clone(),
                        pos: o.vertices.clone(),
                        indices: o
                            .geometry
                            .iter()
                            .flat_map(|g| &g.shapes)
                            .flat_map(|s| match s.primitive {
                                Primitive::Triangle(v1, v2, v3) => [
                                    v1.0.try_into().unwrap(),
                                    v2.0.try_into().unwrap(),
                                    v3.0.try_into().unwrap(),
                                ],
                                _ => todo!("Graceful error"),
                            })
                            .collect(),
                    })
                    .collect(),
            })
        })
    }
}
