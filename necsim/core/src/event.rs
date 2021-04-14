use serde::{Deserialize, Serialize};

use core::{
    cmp::{Ord, Ordering},
    hash::{Hash, Hasher},
};

use crate::{landscape::IndexedLocation, lineage::GlobalLineageReference};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, TypeLayout, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub struct PackedEvent {
    pub origin: IndexedLocation,
    pub time: f64,
    pub global_lineage_reference: GlobalLineageReference,
    pub r#type: EventType,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub enum EventType {
    Speciation,
    Dispersal(Dispersal),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub struct Dispersal {
    pub target: IndexedLocation,
    pub interaction: LineageInteraction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub enum LineageInteraction {
    None,
    Maybe,
    Coalescence(GlobalLineageReference),
}

impl From<Option<GlobalLineageReference>> for LineageInteraction {
    fn from(value: Option<GlobalLineageReference>) -> Self {
        match value {
            None => Self::None,
            Some(lineage) => Self::Coalescence(lineage),
        }
    }
}

#[allow(dead_code)]
const EXCESSIVE_INTERACTION_ERROR: [(); 8] = [(); core::mem::size_of::<LineageInteraction>()];

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub struct SpeciationEvent {
    pub origin: IndexedLocation,
    pub time: f64,
    pub global_lineage_reference: GlobalLineageReference,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub struct DispersalEvent {
    pub origin: IndexedLocation,
    pub time: f64,
    pub global_lineage_reference: GlobalLineageReference,
    pub target: IndexedLocation,
    pub interaction: LineageInteraction,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone /* , PartialEq, Eq */)]
pub enum TypedEvent {
    Speciation(SpeciationEvent),
    Dispersal(DispersalEvent),
}

impl From<SpeciationEvent> for PackedEvent {
    fn from(event: SpeciationEvent) -> Self {
        Self {
            origin: event.origin,
            time: event.time,
            global_lineage_reference: event.global_lineage_reference,
            r#type: EventType::Speciation,
        }
    }
}

impl From<DispersalEvent> for PackedEvent {
    fn from(event: DispersalEvent) -> Self {
        Self {
            origin: event.origin,
            time: event.time,
            global_lineage_reference: event.global_lineage_reference,
            r#type: EventType::Dispersal(Dispersal {
                target: event.target,
                interaction: event.interaction,
            }),
        }
    }
}

impl From<TypedEvent> for PackedEvent {
    fn from(event: TypedEvent) -> Self {
        match event {
            TypedEvent::Speciation(event) => event.into(),
            TypedEvent::Dispersal(event) => event.into(),
        }
    }
}

impl From<PackedEvent> for TypedEvent {
    fn from(event: PackedEvent) -> Self {
        match event.r#type {
            EventType::Speciation => Self::Speciation(SpeciationEvent {
                origin: event.origin,
                time: event.time,
                global_lineage_reference: event.global_lineage_reference,
            }),
            EventType::Dispersal(Dispersal {
                target,
                interaction,
            }) => Self::Dispersal(DispersalEvent {
                origin: event.origin,
                time: event.time,
                global_lineage_reference: event.global_lineage_reference,
                target,
                interaction,
            }),
        }
    }
}

impl Eq for PackedEvent {}

impl PartialEq for PackedEvent {
    // `Event`s are equal when they have the same `origin`, `time` and `r#type`
    // (`global_lineage_reference` is ignored)
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin && self.time == other.time && self.r#type == other.r#type
    }
}

impl Ord for PackedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order `Event`s in lexicographical order:
        //  (1) time
        //  (2) origin
        //  (3) r#type (target)
        match self.time.total_cmp(&other.time) {
            Ordering::Equal => (&self.origin, &self.r#type, &self.global_lineage_reference).cmp(&(
                &other.origin,
                &other.r#type,
                &other.global_lineage_reference,
            )),
            ordering => ordering,
        }
    }
}

impl PartialOrd for PackedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for PackedEvent {
    // `Event`s are equal when they have the same `origin`, `time` and `r#type`
    // (`global_lineage_reference` is ignored)
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.origin.hash(state);
        self.time.to_bits().hash(state);
        self.r#type.hash(state);
    }
}

impl Eq for SpeciationEvent {}

impl PartialEq for SpeciationEvent {
    // `SpeciationEvent`s are equal when they have the same `origin` and `time`
    // (`global_lineage_reference` is ignored)
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin && self.time == other.time
    }
}

impl Eq for DispersalEvent {}

impl PartialEq for DispersalEvent {
    // `SpeciationEvent`s are equal when they have the same `origin`, `time`,
    // `target` and `interaction` (`global_lineage_reference` is ignored)
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin
            && self.time == other.time
            && self.target == other.target
            && self.interaction == other.interaction
    }
}
