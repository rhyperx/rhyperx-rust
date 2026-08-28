use std::marker::PhantomData;

use ouroboros::self_referencing;
use rkyv::bytecheck::CheckBytes;
use rkyv::de::Pool;
use rkyv::rancor::Strategy;
use rkyv::util::AlignedVec;
use rkyv::validation::Validator;
use rkyv::validation::archive::ArchiveValidator;
use rkyv::validation::shared::SharedValidator;
use rkyv::{Archive, Deserialize};
use std::cmp::Eq;
use std::error::Error;
use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::path::Path;

use crate::error::SerializationError;
use crate::hypergraph::hyperedge_container::HyperedgeContainer;
use crate::hypergraph::hypergraph::Hypergraph;
use crate::types::NodeId;

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

// ── Archived handle for Hypergraph ───────────────

#[self_referencing]
pub struct HypergraphHandle<T, W, C>
where
    T: NodeId + Archive + Hash + Eq + 'static,
    W: Archive + 'static,
    C: HyperedgeContainer<T, W> + Archive + 'static,
    <T as Archive>::Archived: Hash + Eq,
{
    bytes: AlignedVec,
    #[borrows(bytes)]
    pub archived: &'this rkyv::Archived<Hypergraph<T, W, C>>,
    _phantom: PhantomData<(T, W, C)>,
}

impl<T, W, C> DumpCacheToFile for Hypergraph<T, W, C>
where
    T: NodeId + Archive + StdSerializable + Hash + Eq,
    <T as Archive>::Archived: Hash + Eq,
    W: StdSerializable,
    C: HyperedgeContainer<T, W> + Archive + StdSerializable + 'static,
{
    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SerializationError> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)?;

        let mut file = File::create(path)?;
        file.write_all(&bytes)?;

        Ok(())
    }
}

impl<T, W, C> LoadFromCacheDeserialized for Hypergraph<T, W, C>
where
    T: NodeId + Hash + Eq + Archive,
    W: Archive,
    C: HyperedgeContainer<T, W> + Archive,
    <T as Archive>::Archived: Hash + Eq,
    for<'a> <T as Archive>::Archived: StdCheckBytes<'a>,
    for<'a> <W as Archive>::Archived: StdCheckBytes<'a>,
    for<'a> <C as Archive>::Archived: StdCheckBytes<'a>,
    <T as Archive>::Archived: StdDeserializable<T>,
    <W as Archive>::Archived: StdDeserializable<W>,
    <C as Archive>::Archived: StdDeserializable<C>,
{
    fn load_deserialized<P: AsRef<Path>>(path: P) -> Result<Self, SerializationError> {
        let mut file = File::open(path)?;

        let mut bytes: AlignedVec = AlignedVec::new();
        bytes.extend_from_reader(&mut file)?;

        let archived =
            rkyv::access::<rkyv::Archived<Hypergraph<T, W, C>>, rkyv::rancor::Error>(&bytes[..])?;
        let rv = rkyv::deserialize::<Hypergraph<T, W, C>, rkyv::rancor::Error>(archived)?;

        Ok(rv)
    }
}

impl<T, W, C> LoadFromCacheArchived for Hypergraph<T, W, C>
where
    T: NodeId + Archive + Hash + Eq + 'static,
    for<'a> <T as Archive>::Archived: StdCheckBytes<'a> + StdDeserializable<T> + Hash + Eq,
    W: Archive + 'static,
    for<'a> <W as Archive>::Archived: StdCheckBytes<'a> + StdDeserializable<W>,
    C: HyperedgeContainer<T, W> + Archive + 'static,
    for<'a> <C as Archive>::Archived: StdCheckBytes<'a> + StdDeserializable<C>,
{
    type Container = HypergraphHandle<T, W, C>;

    fn load_archived<P: AsRef<Path>>(path: P) -> Result<Self::Container, SerializationError> {
        let mut file = File::open(path)?;
        let mut bytes = AlignedVec::new();
        bytes.extend_from_reader(&mut file)?;

        let container = HypergraphHandleTryBuilder {
            bytes,
            archived_builder: |bytes_ref| {
                rkyv::access::<rkyv::Archived<Hypergraph<T, W, C>>, rkyv::rancor::Error>(
                    &bytes_ref[..],
                )
                .map_err(|e| Box::new(e) as Box<dyn Error>)
            },
            _phantom: PhantomData,
        }
        .try_build()
        .map_err(|e| SerializationError::Unknown(e.to_string()))?;

        Ok(container)
    }
}
