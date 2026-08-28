use ouroboros::self_referencing;
use rkyv::bytecheck::CheckBytes;
use rkyv::de::Pool;
use rkyv::rancor::Strategy;
use rkyv::util::AlignedVec;
use rkyv::validation::Validator;
use rkyv::validation::archive::ArchiveValidator;
use rkyv::validation::shared::SharedValidator;
use rkyv::{Archive, Deserialize};
use rust_core_macros::hoist_mod;
use std::cmp::Eq;
use std::fs::File;
use std::hash::Hash;
use std::path::Path;
use std::{error::Error, io::Write};

use crate::misc::error::SerializationError;
use crate::types::hypergraph::hypergraph::{ArchivedHypergraph, Hypergraph};
use crate::types::{ArchivedHx, UnweightedHypergraph, WeightedHypergraph};

pub trait StdSerializable:
    for<'a> rkyv::Serialize<
        rkyv::rancor::Strategy<
            rkyv::ser::Serializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::ser::sharing::Share,
            >,
            rkyv::rancor::Error,
        >,
    >
{
}
impl<T> StdSerializable for T where
    T: for<'a> rkyv::Serialize<
            rkyv::rancor::Strategy<
                rkyv::ser::Serializer<
                    rkyv::util::AlignedVec,
                    rkyv::ser::allocator::ArenaHandle<'a>,
                    rkyv::ser::sharing::Share,
                >,
                rkyv::rancor::Error,
            >,
        >
{
}

pub trait StdDeserializable<T>: Deserialize<T, Strategy<Pool, rkyv::rancor::Error>> {}
impl<T, U> StdDeserializable<T> for U where U: Deserialize<T, Strategy<Pool, rkyv::rancor::Error>> {}

pub trait StdCheckBytes<'a>:
    CheckBytes<Strategy<Validator<ArchiveValidator<'a>, SharedValidator>, rkyv::rancor::Error>>
{
}

impl<'a, T> StdCheckBytes<'a> for T where
    T: CheckBytes<Strategy<Validator<ArchiveValidator<'a>, SharedValidator>, rkyv::rancor::Error>>
{
}

pub trait DumpCacheToFile {
    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SerializationError>;
}

pub trait LoadFromCacheDeserialized: Sized {
    fn load_deserialized<P: AsRef<Path>>(path: P) -> Result<Self, SerializationError>;
}

pub trait LoadFromCacheArchived: Sized {
    type Container;

    fn load_archived<P: AsRef<Path>>(path: P) -> Result<Self::Container, SerializationError>;
}

impl<const N: usize, T, W> Hash for ArchivedHx<N, T, W>
where
    T: Hash + Archive,
    W: Archive,
    <T as Archive>::Archived: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nodes.hash(state);
    }
}

impl<const N: usize, T, W> PartialEq for ArchivedHx<N, T, W>
where
    T: Archive,
    <T as Archive>::Archived: PartialEq,
    W: Archive,
{
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<const N: usize, T, W> Eq for ArchivedHx<N, T, W>
where
    T: Archive,
    <T as Archive>::Archived: Eq,
    W: Archive,
{
}

#[self_referencing]
pub struct ArchivedHypergraphHandle<T, W>
where
    T: Archive + Hash + Eq + 'static,
    W: Archive + 'static,
    <T as Archive>::Archived: Hash + Eq,
{
    bytes: AlignedVec,
    #[borrows(bytes)]
    // #[not_covariant]
    pub archived: &'this super::ArchivedHypergraph<T, W>,
}

impl<T, W> DumpCacheToFile for Hypergraph<T, W>
where
    T: Archive + StdSerializable + Hash + Eq,
    <T as Archive>::Archived: Hash + Eq,
    W: StdSerializable,
{
    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SerializationError> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)?;

        let mut file = File::create(path)?;
        file.write_all(&bytes)?;

        Ok(())
    }
}

impl<T, W> LoadFromCacheDeserialized for Hypergraph<T, W>
where
    T: Hash + Eq + Archive,
    W: rkyv::Archive,
    <T as Archive>::Archived: Hash + Eq,
    for<'a> <T as Archive>::Archived: StdCheckBytes<'a>,
    for<'a> <W as Archive>::Archived: StdCheckBytes<'a>,
    <T as Archive>::Archived: StdDeserializable<T>,
    <W as Archive>::Archived: StdDeserializable<W>,
{
    fn load_deserialized<P: AsRef<Path>>(path: P) -> Result<Self, SerializationError> {
        let mut file = File::open(path)?;

        // Fix: Explicitly type your AlignedVec if the compiler gets confused
        let mut bytes: AlignedVec = AlignedVec::new();
        bytes.extend_from_reader(&mut file)?;

        let archived = rkyv::access::<ArchivedHypergraph<T, W>, rkyv::rancor::Error>(&bytes[..])?;
        let rv = rkyv::deserialize::<Hypergraph<T, W>, rkyv::rancor::Error>(archived)?;

        Ok(rv)
    }
}

impl<T, W> LoadFromCacheArchived for Hypergraph<T, W>
where
    T: Archive + Hash + Eq + 'static,
    for<'a> <T as Archive>::Archived: StdCheckBytes<'a> + StdDeserializable<T> + Hash + Eq,
    W: Archive + 'static,
    for<'a> <W as Archive>::Archived: StdCheckBytes<'a> + StdDeserializable<W>,
{
    type Container = ArchivedHypergraphHandle<T, W>;

    fn load_archived<P: AsRef<Path>>(path: P) -> Result<Self::Container, SerializationError> {
        let mut file = File::open(path)?;
        let mut bytes = AlignedVec::new();
        bytes.extend_from_reader(&mut file)?;

        let container = ArchivedHypergraphHandleTryBuilder {
            bytes,
            archived_builder: |bytes_ref| {
                rkyv::access::<ArchivedHypergraph<T, W>, rkyv::rancor::Error>(&bytes_ref[..])
                    .map_err(|e| Box::new(e) as Box<dyn Error>)
            },
        }
        .try_build()
        .map_err(|e| SerializationError::Unknown(e.to_string()))?;

        Ok(container)
    }
}

// Delegate cache (de)serialization for the public wrapper hypergraph types so they can be
// used directly as Loader::Output in loader implementations.
impl DumpCacheToFile for UnweightedHypergraph {
    fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), SerializationError> {
        self.0.save_to_file(path)
    }
}

impl LoadFromCacheDeserialized for UnweightedHypergraph {
    fn load_deserialized<P: AsRef<std::path::Path>>(path: P) -> Result<Self, SerializationError> {
        let hg: Hypergraph<_, _> = Hypergraph::load_deserialized(path)?;
        Ok(hg.into())
    }
}

impl DumpCacheToFile for WeightedHypergraph {
    fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), SerializationError> {
        self.0.save_to_file(path)
    }
}

impl LoadFromCacheDeserialized for WeightedHypergraph {
    fn load_deserialized<P: AsRef<std::path::Path>>(path: P) -> Result<Self, SerializationError> {
        let hg: Hypergraph<_, _> = Hypergraph::load_deserialized(path)?;
        Ok(hg.into())
    }
}

// #[self_referencing]
// pub struct ArchivedAdjListHandle<W>
// where
//     W: Archive + 'static,
// {
//     bytes: AlignedVec,
//     #[borrows(bytes)]
//     // #[not_covariant]
//     pub archived: &'this ArchivedAdjList<W>,
// }
// impl<W> DumpCacheToFile for AdjList<W>
// where
//     W: StdSerializable,
// {
//     fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), SerializationError> {
//         let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)?;
//
//         let mut file = File::create(path)?;
//         file.write_all(&bytes)?;
//
//         Ok(())
//     }
// }
//
// impl<W> LoadFromCacheDeserialized for AdjList<W>
// where
//     W: StdDeserializable<W> + Archive,
//     for<'a> <W as Archive>::Archived: StdCheckBytes<'a> + StdDeserializable<W>,
// {
//     fn load_deserialized<P: AsRef<std::path::Path>>(path: P) -> Result<Self, SerializationError> {
//         let mut file = File::open(path)?;
//
//         let mut bytes: AlignedVec = AlignedVec::new();
//         bytes.extend_from_reader(&mut file)?;
//
//         let archived = rkyv::access::<ArchivedAdjList<W>, rkyv::rancor::Error>(&bytes[..])?;
//         let rv = rkyv::deserialize::<AdjList<W>, rkyv::rancor::Error>(archived)?;
//
//         Ok(rv)
//     }
// }
//
// impl<W> LoadFromCacheArchived for AdjList<W>
// where
//     W: Archive + 'static,
//     for<'a> <W as Archive>::Archived: StdCheckBytes<'a> + StdDeserializable<W>,
// {
//     type Container = ArchivedAdjListHandle<W>;
//
//     fn load_archived<P: AsRef<Path>>(path: P) -> Result<Self::Container, SerializationError> {
//         let mut file = File::open(path)?;
//         let mut bytes = AlignedVec::new();
//         bytes.extend_from_reader(&mut file)?;
//
//         let container = ArchivedAdjListHandleTryBuilder {
//             bytes,
//             archived_builder: |bytes_ref| {
//                 rkyv::access::<ArchivedAdjList<W>, rkyv::rancor::Error>(&bytes_ref[..])
//                     .map_err(|e| Box::new(e) as Box<dyn Error>)
//             },
//         }
//         .try_build()
//         .map_err(|e| SerializationError::Unknown(e.to_string()))?;
//
//         Ok(container)
//     }
// }
