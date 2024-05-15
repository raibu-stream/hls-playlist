//! TODO: Document

// Copyright 2024 Logan Wemyss
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{collections::HashMap, io};

use serde::ser::Serializer;
use thiserror::Error;

/// TODO: Document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteeringManifest {
    ttl_seconds: u64,
    reload_uri: Option<String>,
    pathway_priority: Vec<u64>,
    pathway_clones: Vec<PathwayClone>,
}

/// TODO: Document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathwayClone {
    base_id: String,
    id: String,
    uri_replacement: UriReplacement,
}

/// TODO: Document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriReplacement {
    host: Option<String>,
    query_parameters: Option<HashMap<String, String>>,
    per_variant_uris: Option<HashMap<String, String>>,
    per_rendition_uris: Option<HashMap<String, String>>,
}

impl SteeringManifest {
    /// Serializes the manifest into it's json representation.
    ///
    /// # Errors
    ///
    /// May return `Err` when encountering an io error on `output`.
    pub fn serialize(&self, output: impl io::Write) -> Result<(), SerializeError> {
        let mut serializer = serde_json::Serializer::new(output);

        serializer.serialize_u64(1)?;

        todo!()
    }
}

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
