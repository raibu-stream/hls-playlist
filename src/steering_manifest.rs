//! A representation of a HLS steering manifest.
//!
//! Content steering allows content producers to group redundant
//! variant streams into "pathways" and to dynamically prioritize
//! access to different pathways.

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

use std::collections::HashMap;
use std::collections::HashSet;
use std::io;

use serde::ser::SerializeStruct;
use serde::ser::Serializer;
use serde::Serialize;

/// A steering manifest which identifies the available pathways
/// and their priority order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteeringManifest {
    /// Specifies how many seconds the client must wait before
    /// reloading the Steering Manifest.
    pub ttl_seconds: u64,

    /// Specifies the URI the client must use the
    /// next time it obtains the Steering Manifest.
    pub reload_uri: Option<String>,

    /// A list of pathway IDs order to most preferred to least preferred.
    pub pathway_priority: HashSet<String>,

    /// A list of novel pathways made by cloning existing ones.
    pub pathway_clones: Vec<PathwayClone>,
}

/// A way to introduce novel Pathways by cloning existing ones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathwayClone {
    /// The ID of the base pathway, which this clone is based on.
    pub base_id: String,

    /// The ID of this new pathway.
    pub id: String,

    /// URI Replacement rules.
    pub uri_replacement: UriReplacement,
}

/// URI replacement rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriReplacement {
    /// If Some, replace the hostname of every rendition URI
    /// in the new pathway.
    pub host: Option<String>,

    /// URI params to append to every rendition URI in the new
    /// pathway.
    pub query_parameters: Option<HashMap<String, String>>,

    /// If the `stable_variant_id` of a `VariantStream` on the new
    /// pathway appears in the map, set its URI to be the entry's value.
    pub per_variant_uris: Option<HashMap<String, String>>,

    /// Key value pairs. If the `stable_rendition_id` of a rendition referred to by a
    /// `VariantStream` on the new pathway appears in the map, set
    /// its URI to be the entry's value.
    pub per_rendition_uris: Option<HashMap<String, String>>,
}

impl SteeringManifest {
    /// Serializes the manifest into it's json representation.
    /// Guaranteed to write valid UTF-8 only.
    ///
    /// This does not percent encode [`UriReplacement::query_parameters`].
    ///
    /// # Errors
    ///
    /// May return `Err` when encountering an io error on `output`.
    ///
    /// # Panics
    ///
    /// A number of invariants mirroring the requirements of the HLS spec
    /// must be upheld when making a call to this method, or else it may
    /// panic. They are listed below.
    ///
    /// * [`SteeringManifest::pathway_priority`] must be non-empty.
    ///
    /// * [`UriReplacement::host`], if `Some`, must be non-empty.
    ///
    /// * [`UriReplacement::query_parameters`] must not contain a key which is empty.
    pub fn serialize(&self, output: impl io::Write) -> Result<(), serde_json::Error> {
        serde_json::to_writer(output, self)
    }
}

impl Serialize for SteeringManifest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len_of_fields = 3;
        if self.reload_uri.is_some() {
            len_of_fields += 1;
        }
        if !self.pathway_clones.is_empty() {
            len_of_fields += 1;
        }

        let mut manifest = serializer.serialize_struct("SteeringManifest", len_of_fields)?;
        manifest.serialize_field("VERSION", &1)?;
        manifest.serialize_field("TTL", &self.ttl_seconds)?;
        if let Some(reload_uri) = &self.reload_uri {
            manifest.serialize_field("RELOAD-URI", reload_uri)?;
        }

        assert!(
            !self.pathway_priority.is_empty(),
            "Found an empty pathway priority list while serializing."
        );
        manifest.serialize_field("PATHWAY-PRIORITY", &self.pathway_priority)?;

        if self.pathway_clones.is_empty() {
            return manifest.end();
        }
        manifest.serialize_field("PATHWAY-CLONES", &self.pathway_clones)?;

        manifest.end()
    }
}

impl Serialize for PathwayClone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut clone = serializer.serialize_struct("PathwayClone", 3)?;

        clone.serialize_field("BASE-ID", &self.base_id)?;
        clone.serialize_field("ID", &self.id)?;
        clone.serialize_field("URI-REPLACEMENT", &self.uri_replacement)?;
        clone.end()
    }
}

impl Serialize for UriReplacement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len_of_fields = 0;
        if self.host.is_some() {
            len_of_fields += 1;
        }
        if self.query_parameters.is_some() {
            len_of_fields += 1;
        }
        if self.per_variant_uris.is_some() {
            len_of_fields += 1;
        }
        if self.per_rendition_uris.is_some() {
            len_of_fields += 1;
        }
        let mut replacement = serializer.serialize_struct("UriReplacement", len_of_fields)?;

        if let Some(host) = &self.host {
            assert!(
                !host.is_empty(),
                "Found an empty host string while serializing."
            );

            replacement.serialize_field("HOST", host)?;
        }

        if let Some(params) = &self.query_parameters {
            assert!(
                !params.contains_key(""),
                "Found an empty query parameter key while serializing."
            );

            replacement.serialize_field("PARAMS", params)?;
        }

        if let Some(uris) = &self.per_variant_uris {
            replacement.serialize_field("PER-VARIANT-URIS", uris)?;
        }

        if let Some(uris) = &self.per_rendition_uris {
            replacement.serialize_field("PER-RENDITION-URIS", uris)?;
        }

        replacement.end()
    }
}
