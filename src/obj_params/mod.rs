/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fs::{read_dir, File};
use std::io::BufReader;
use std::sync::Arc;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use serde_json;
use serde::{Deserialize, Deserializer, de::{self, Visitor}};
use crate::{
  addr::{Addr, AddrOffsets},
  big_endian::DecodeBE,
  dolphin::Dolphin,
  sms::SMSDolphin,
};

mod field_reader;
use field_reader::*;

/**** original json ****/
#[derive(Debug, Deserialize)]
struct ObjParamsJson {
  offsets: Box<[ObjParamsOffsetEntry]>,
}

#[derive(Debug, Deserialize)]
struct ObjParamsOffsetEntry {
  #[serde(deserialize_with = "deserialize_obj_params_offset_entry")]
  offset: AddrOffsets,
  #[serde(rename = "type")]
  type_: Arc<str>,
  name: Arc<str>,
  notes: Arc<str>,
  #[serde(
    default = "ObjParamsOffsetEntryFormat::none",
    deserialize_with = "deserialize_obj_params_offset_entry_format",
  )]
  format: Option<ObjParamsOffsetEntryFormat>,
  hidden: Option<bool>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ObjParamsOffsetEntryFormat {
  Hex,
}
impl ObjParamsOffsetEntryFormat {
  #[inline]
  fn none() -> Option<ObjParamsOffsetEntryFormat> {
    None
  }
}

impl std::fmt::Display for ObjParamsOffsetEntryFormat {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    match self {
      Self::Hex => write!(fmt, "hex"),
    }
  }
}

fn deserialize_obj_params_offset_entry<'de, D>(deserializer: D) -> Result<AddrOffsets, D::Error>
where
  D: Deserializer<'de>,
{
  struct ValueVisitor;
  impl<'de> Visitor<'de> for ValueVisitor {
    type Value = AddrOffsets;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a hex string or a non-empty array of hex string")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
      E: de::Error,
    {
      u32::from_str_radix(value, 16)
        .map_err(|e| E::custom(e))
        .map(|x| AddrOffsets(x, Box::from([])))
    }
    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
      S: de::SeqAccess<'de>,
    {
      let mut arr = match seq.size_hint() {
        Some(size) => Vec::<u32>::with_capacity(size),
        None => Vec::<u32>::new(),
      };
      while let Some(value) = seq.next_element::<Cow<'de, str>>()? {
        arr.push(u32::from_str_radix(&value, 16).map_err(de::Error::custom)?);
      }
      arr.split_first()
        .map(|p| AddrOffsets(*p.0, p.1.into()))
        .ok_or_else(|| de::Error::custom("Offset array must not be empty"))
    }
  }

  deserializer.deserialize_any(ValueVisitor)
}

fn deserialize_obj_params_offset_entry_format<'de, D>(deserializer: D)
  -> Result<Option<ObjParamsOffsetEntryFormat>, D::Error>
where
  D: Deserializer<'de>,
{
  struct ValueVisitor;
  impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Option<ObjParamsOffsetEntryFormat>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("\"hex\" or omitted")
    }
    fn visit_none<E>(self) -> Result<Self::Value, E> {
      Ok(None)
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
      E: de::Error,
    {
      match value {
        "hex" => Ok(Some(ObjParamsOffsetEntryFormat::Hex)),
        _ => Err(E::unknown_variant(value, &["hex"])),
      }
    }
  }

  deserializer.deserialize_any(ValueVisitor)
}

/**** parsed ****/
pub enum ObjectType<D: Dolphin> {
  Primitive(ClassFieldReader<D>),
  Class(Box<[ClassField<D>]>),
}
impl<D: Dolphin> ObjectType<D> {
  fn new_primitive<T: DecodeBE + ToString + Send + Sync + 'static>() -> Self {
    ObjectType::<D>::Primitive(Arc::new(PrimitiveFieldReader::<T>::new()))
  }
}

type ClassFieldReader<D> = Arc<dyn FieldReader<D, String> + Send + Sync>;
pub struct ClassField<D: Dolphin> {
  pub offset: AddrOffsets,
  pub type_: Arc<str>,
  pub name: Arc<str>,
  pub notes: Arc<str>,
  pub class: Arc<str>,
  pub reader: ClassFieldReader<D>,
}
impl<D: Dolphin> FieldReader<D, String> for ClassField<D> {
  #[inline]
  fn read(&self, d: &D, addr: Addr) -> Option<String> {
    self.reader.read(d, addr)
  }
}

pub type ObjParams<D> = HashMap<Arc<str>, ObjectType<D>>;
pub type ObjParamsLoadResult<D> = Result<ObjParams<D>, std::io::Error>;
pub fn load_obj_params(dir: &Path) -> ObjParamsLoadResult<SMSDolphin> {
  type D = SMSDolphin; // TODO
  let entry_reader = read_dir(dir)?;
  let mut db_raw = HashMap::<Arc<str>, Arc<ObjParamsJson>>::new();
  entry_reader.for_each(|entry| {
    let Ok(entry) = entry.map_err(|e| eprintln!("Fail to get entry: {e}")) else {return};
    let path = entry.path();
    if Some(true) != path.extension().map(|e| e == "json") {return}
    let Ok(file) = File::open(&path)
      .map_err(|e| eprintln!("Fail to open file \"{}\": {e}", path.to_string_lossy())) else {return};
    let reader = BufReader::new(file);
    let Ok(o) = serde_json::from_reader::<_, HashMap<Arc<str>, Arc<ObjParamsJson>>>(reader)
      .map_err(|e| eprintln!("Fail to parse {}: {e}", path.to_string_lossy())) else {return};
    for e in o {
      db_raw.insert(e.0.clone(), e.1.clone());
    }
  });

  struct Env<'a, D: Dolphin> {
    db_raw: &'a HashMap::<Arc<str>, Arc<ObjParamsJson>>,
    db_types: HashMap::<Arc<str>, ObjectType<D>>,
    db_formatted: HashMap::<(&'a str, ObjParamsOffsetEntryFormat), ClassFieldReader<D>>,
    reader_unk: ClassFieldReader<D>,
    type_addr: ObjectType<D>,
  }
  fn resolve_type<'a, D: Dolphin>(env: &'a mut Env<D>, type_: Arc<str>) -> &'a ObjectType<D> {
    if !env.db_types.contains_key(&type_) {
      if type_.ends_with('*') {
        return &env.type_addr;
      }
      let new_type = match env.db_raw.get(&type_) {
        Some(o) => {
          let mut class_fields = Vec::<ClassField<D>>::new();
          for field in o.offsets.iter() {
            // skip hidden fields
            if let Some(true) = field.hidden {continue}
            // format
            if let Some(format) = field.format {
              if let Some(reader) = env.db_formatted.get(&(&field.type_, format)) {
                class_fields.push(
                  ClassField {
                    reader: reader.clone(),
                    offset: field.offset.clone(),
                    name: field.name.clone(),
                    notes: field.notes.clone(),
                    type_: field.type_.clone(),
                    class: type_.clone(),
                  },
                );
                continue;
              } else {
                eprintln!("format \"{format}\" cannot be used for type \"{}\" (in class \"{type_}\")", field.type_);
              }
            }
            // resolve
            match resolve_type(env, field.type_.clone()) {
              ObjectType::<D>::Primitive(reader) => class_fields.push(
                ClassField {
                  reader: reader.clone(),
                  offset: field.offset.clone(),
                  name: field.name.clone(),
                  notes: field.notes.clone(),
                  type_: field.type_.clone(),
                  class: type_.clone(),
                },
              ),
              ObjectType::<D>::Class(subfields) => {
                let is_name_template = field.name.contains('*');
                for subfield in subfields.iter() {
                  class_fields.push(ClassField {
                    reader: subfield.reader.clone(),
                    offset: &field.offset + &subfield.offset,
                    name: match is_name_template {
                      true => Arc::from(field.name.replace('*', &subfield.name)),
                      false => subfield.name.clone(),
                    },
                    notes: subfield.notes.clone(),
                    type_: subfield.type_.clone(),
                    class: subfield.class.clone(),
                  })
                }
              },
            };
          }
          ObjectType::<D>::Class(class_fields.into())
        },
        None => {
          ObjectType::<D>::Primitive(env.reader_unk.clone())
        },
      };
      env.db_types.insert(type_.clone(), new_type);
    }
    env.db_types.get(&type_).unwrap()
  }

  let mut env = Env {
    db_raw: &db_raw,
    db_types: HashMap::<Arc<str>, ObjectType<D>>::from([
      (Arc::from("u8"), ObjectType::<D>::new_primitive::<u8>()),
      (Arc::from("u16"), ObjectType::<D>::new_primitive::<u16>()),
      (Arc::from("u32"), ObjectType::<D>::new_primitive::<u32>()),
      (Arc::from("s8"), ObjectType::<D>::new_primitive::<i8>()),
      (Arc::from("s16"), ObjectType::<D>::new_primitive::<i16>()),
      (Arc::from("s32"), ObjectType::<D>::new_primitive::<i32>()),
      (Arc::from("float"), ObjectType::<D>::Primitive(Arc::new(F32FieldReader))),
      (Arc::from("string"), ObjectType::<D>::Primitive(Arc::new(StringFieldReader))),
      (Arc::from("void*"), ObjectType::<D>::Primitive(Arc::new(ClassNameReader))),
    ]),
    db_formatted: HashMap::from([
      (
        ("u8", ObjParamsOffsetEntryFormat::Hex),
        Arc::new(HexFieldReader(1)) as ClassFieldReader<D>,
      ),
      (
        ("u16", ObjParamsOffsetEntryFormat::Hex),
        Arc::new(HexFieldReader(2)) as ClassFieldReader<D>,
      ),
      (
        ("u32", ObjParamsOffsetEntryFormat::Hex),
        Arc::new(HexFieldReader(4)) as ClassFieldReader<D>,
      ),
    ]),
    reader_unk: Arc::new(PrimitiveFieldReader::<Addr>::new()), // TODO
    type_addr: ObjectType::<D>::new_primitive::<Addr>(),
  };
  for type_ in db_raw.keys() {
    resolve_type(&mut env, type_.clone());
  }
  Ok(env.db_types)
}
