use std::{
    array::TryFromSliceError,
    convert::{TryFrom, TryInto},
    io,
};

use necsim_core_bond::NonZeroOneU64;
use tskit::metadata::{MetadataError, MetadataRoundtrip};

use necsim_core::lineage::GlobalLineageReference;

#[allow(clippy::module_name_repetitions)]
#[repr(transparent)]
pub struct GlobalLineageMetadata(GlobalLineageReference);

impl MetadataRoundtrip for GlobalLineageMetadata {
    fn encode(&self) -> Result<Vec<u8>, MetadataError> {
        // Store the internal u64 without the +2 offset
        Ok((unsafe { self.0.clone().into_inner() }.get() - 2)
            .to_le_bytes()
            .to_vec())
    }

    fn decode(metadata: &[u8]) -> Result<Self, MetadataError>
    where
        Self: Sized,
    {
        // Ensure that `metadata` contains exactly eight bytes
        let value_bytes: [u8; 8] = metadata.try_into().map_err(|err: TryFromSliceError| {
            MetadataError::RoundtripError {
                value: Box::new(io::Error::new(io::ErrorKind::InvalidData, err.to_string())),
            }
        })?;

        // Convert the bytes into an u64 with the needed +2 offset
        let value = u64::from_le_bytes(value_bytes) + 2;

        // Create the internal `NonZeroOneU64` representation of the reference
        let value_inner =
            NonZeroOneU64::try_from(value).map_err(|err| MetadataError::RoundtripError {
                value: Box::new(io::Error::new(io::ErrorKind::InvalidData, err.to_string())),
            })?;

        Ok(Self(unsafe {
            GlobalLineageReference::from_inner(value_inner)
        }))
    }
}

impl GlobalLineageMetadata {
    pub fn new(reference: &GlobalLineageReference) -> &Self {
        unsafe { &*(reference as *const GlobalLineageReference).cast() }
    }
}
