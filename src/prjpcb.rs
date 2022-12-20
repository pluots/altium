mod parser;

use crate::common::UniqueId;
use crate::errors::AltiumError;
use ini::{Ini, Properties};
use lazy_static::lazy_static;
use parser::{parse_bool, parse_int, parse_string};
use regex::Regex;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use uuid::Uuid;

use self::parser::parse_unique_id;

lazy_static! {
    static ref DOC_RE: Regex = Regex::new(r"Document\d+").unwrap();
}

#[non_exhaustive]
pub struct PrjPcb {
    design: Design,
    preferences: Option<Preferences>,
    release: Option<Release>,
    documents: Vec<Document>,
    variants: Vec<Variant>,
    parameters: Vec<HashMap<String, String>>,
    configurations: Vec<Configuration>,
    original: Ini,
}

impl Debug for PrjPcb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrjPcb")
            .field("design", &self.design)
            .field("preferences", &self.preferences)
            .field("release", &self.release)
            .field("documents", &self.documents)
            .field("variants", &self.variants)
            .field("parameters", &self.parameters)
            .field("configurations", &self.configurations)
            .finish()
    }
}

impl PrjPcb {
    pub fn design(&self) -> &Design {
        &self.design
    }
    pub fn preferences(&self) -> Option<&Preferences> {
        self.preferences.as_ref()
    }
    pub fn release(&self) -> Option<&Release> {
        self.release.as_ref()
    }
    pub fn documents(&self) -> &Vec<Document> {
        &self.documents
    }

    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Self, AltiumError> {
        let ini = Ini::load_from_file(filename)?;
        Self::from_ini(ini)
    }
    pub fn from_string(s: &str) -> Result<Self, AltiumError> {
        let ini = Ini::load_from_str(s)?;
        Self::from_ini(ini)
    }

    pub fn from_ini(ini: Ini) -> Result<Self, AltiumError> {
        for (i, s) in ini.iter().take(10).enumerate() {
            eprintln!("{i}:\n{s:#?}\n")
        }

        Ok(Self {
            design: Design::from_prj_ini(ini)?,
            preferences: todo!(),
            release: todo!(),
            documents: todo!(),
            variants: todo!(),
            parameters: todo!(),
            configurations: todo!(),
            original: todo!(),
        })
    }
}

/// Design section of a `PrjPCB` file
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct Design {
    original: Properties,
}

impl Design {
    fn from_prj_ini(ini: Ini) -> Result<Self, AltiumError> {
        let sec = ini
            .section(Some("Design"))
            .ok_or(AltiumError::MissingSection("Design".to_owned()))?;

        Ok(Self {
            original: sec.to_owned(),
        })
    }

    fn to_ini(&self) -> &Properties {
        &self.original
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct Preferences {
    original: Properties,
}

impl Preferences {
    fn from_prj_ini(ini: Ini) -> Result<Option<Self>, AltiumError> {
        Ok(ini.section(Some("Preferences")).map(|sec| Self {
            original: sec.to_owned(),
        }))
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct Release {
    original: Properties,
}

impl Release {
    fn from_prj_ini(ini: Ini) -> Result<Self, AltiumError> {
        todo!()
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct Document {
    document_path: String,
    annotation_en: bool,
    annotation_start_value: i32,
    annotation_idx_ctrl_en: bool,
    annotation_suffix: String,
    annotate_order: i32,
    do_libarary_update: bool,
    do_database_update: bool,
    class_gen_cc_auto_en: bool,
    class_gen_cc_auto_room_en: bool,
    class_gen_nc_auto_scope: String,
    item_revision_guid: String,
    generate_class_cluster: bool,
    document_unique_id: UniqueId,
}

impl Document {
    /// Create a vector of `Document`s from an ini file
    fn from_prj_ini(ini: Ini) -> Result<Vec<Self>, AltiumError> {
        let mut doc_sections: Vec<&str> = ini
            .sections()
            .filter_map(|nameopt| {
                nameopt
                    .map(|name| {
                        if DOC_RE.is_match(name) {
                            Some(name)
                        } else {
                            None
                        }
                    })
                    .flatten()
            })
            .collect();

        doc_sections.sort_by_key(|s| s.strip_prefix("Document").unwrap().parse::<i32>().unwrap());

        let mut ret = Vec::new();
        let sec_iter = doc_sections
            .iter()
            .map(|sec_name| ini.section(Some(*sec_name)).unwrap())
            .map(Self::from_section);

        for sec_opt in sec_iter {
            ret.push(sec_opt?)
        }

        Ok(ret)
    }

    /// Create a single `Document` from an ini section
    fn from_section(sec: &Properties) -> Result<Self, AltiumError> {
        Ok(Self {
            document_path: parse_string(sec, "DocumentPath"),
            annotation_en: parse_bool(sec, "AnnotationEnabled"),
            annotation_start_value: parse_int(sec, "AnnotateStartValue"),
            annotation_idx_ctrl_en: parse_bool(sec, "AnnotationIndexControlEnabled"),
            annotation_suffix: parse_string(sec, "AnnotateSuffix"),
            annotate_order: parse_int(sec, "AnnotateScope"),
            do_libarary_update: parse_bool(sec, "DoLibraryUpdate"),
            do_database_update: parse_bool(sec, "DoDatabaseUpdate"),
            class_gen_cc_auto_en: parse_bool(sec, "ClassGenCCAutoEnabled"),
            class_gen_cc_auto_room_en: parse_bool(sec, "ClassGenCCAutoRoomEnabled"),
            class_gen_nc_auto_scope: parse_string(sec, "ClassGenNCAutoScope"),
            item_revision_guid: parse_string(sec, "DItemRevisionGUID"),
            generate_class_cluster: parse_bool(sec, "GenerateClassCluster"),
            document_unique_id: parse_unique_id(sec, "DocumentUniqueId")?,
        })
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct Variant {
    unique_id: Uuid,
    description: String,
    allow_fabrication: bool,
    parameter_count: u32,
    variations: Vec<Variation>,
}

impl Variant {
    fn from_prj_ini(ini: Ini) -> Result<Self, AltiumError> {
        todo!()
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct Variation {}

impl Variation {
    fn from_prj_ini(ini: Ini) -> Result<Self, AltiumError> {
        todo!()
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct Configuration {
    name: String,
    // ConfigurationType should be an enum, seems to match with `ContentTypeGUID`
}

impl Configuration {
    fn from_prj_ini(ini: Ini) -> Result<Self, AltiumError> {
        todo!()
    }
}

#[cfg(test)]
mod tests;
