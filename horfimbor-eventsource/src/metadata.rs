use eventstore::EventData;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;
use thiserror::Error;
use uuid::Uuid;

use crate::{Command, Event};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Metadata {
    #[serde(skip_serializing)]
    id: Option<Uuid>,
    #[serde(rename = "$correlationId")]
    correlation_id: Uuid,
    #[serde(rename = "$causationId")]
    causation_id: Uuid,
    #[serde(rename = "is_event")]
    is_event: bool,
}

impl Metadata {
    #[must_use]
    pub const fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    #[must_use]
    pub const fn causation_id(&self) -> Uuid {
        self.causation_id
    }
    pub fn set_id(&mut self, id: Option<Uuid>) {
        self.id = id;
    }

    #[must_use]
    pub const fn id(&self) -> Option<Uuid> {
        self.id
    }

    #[must_use]
    pub const fn new(
        id: Option<Uuid>,
        correlation_id: Uuid,
        causation_id: Uuid,
        is_event: bool,
    ) -> Self {
        Self {
            id,
            correlation_id,
            causation_id,
            is_event,
        }
    }

    #[must_use]
    pub const fn is_event(&self) -> bool {
        self.is_event
    }
}

#[derive(Clone)]
pub struct CompleteEvent {
    event_data: EventData,
    metadata: Metadata,
}

impl CompleteEvent {
    pub const fn event_data(&self) -> &EventData {
        &self.event_data
    }
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// # Errors
    ///
    /// Will return `Err` if `Metadata` cannot be de into json
    pub fn full_event_data(&self) -> Result<EventData, Error> {
        self.event_data
            .clone()
            .metadata_as_json(self.metadata())
            .map_err(Error::SerdeError)
    }

    /// # Errors
    ///
    /// Will return `Err` if `Metadata` cannot be de into json
    pub fn from_command<C>(command: C, previous_metadata: Option<&Metadata>) -> Result<Self, Error>
    where
        C: Command,
    {
        let event_data =
            EventData::json(command.command_name(), command).map_err(Error::SerdeError)?;

        Ok(Self::from_event_data(event_data, previous_metadata, false))
    }

    /// # Errors
    ///
    /// Will return `Err` if `Metadata` cannot be serialized into json
    pub fn from_event<E>(event: E, previous_metadata: &Metadata) -> Result<Self, Error>
    where
        E: Event,
    {
        let event_data = EventData::json(event.event_name(), event).map_err(Error::SerdeError)?;

        Ok(Self::from_event_data(
            event_data,
            Some(previous_metadata),
            true,
        ))
    }

    fn from_event_data(
        mut event_data: EventData,
        previous_metadata: Option<&Metadata>,
        is_event: bool,
    ) -> Self {
        let id = Uuid::new_v4();

        event_data = event_data.id(id);

        let metadata = previous_metadata.map_or(
            Metadata {
                id: Some(id),
                correlation_id: id,
                causation_id: id,
                is_event,
            },
            |previous| Metadata {
                id: Some(id),
                correlation_id: previous.correlation_id,
                causation_id: previous.id.unwrap_or(id),
                is_event,
            },
        );

        Self {
            event_data,
            metadata,
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("internal `{0}`")]
    SerdeError(SerdeError),
}
