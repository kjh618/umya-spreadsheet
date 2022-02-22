use quick_xml::events::{BytesDecl, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use reader::driver::*;
use std::io;
use std::io::Read;
use structs::raw::RawFile;
use structs::raw::RawRelationship;
use structs::StringValue;
use structs::WriterManager;
use writer::driver::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct RawRelationships {
    file_target: StringValue,
    relationship_list: Vec<RawRelationship>,
}
impl RawRelationships {
    pub(crate) fn _get_file_name(&self) -> String {
        let v: Vec<&str> = self.get_file_target().split('/').collect();
        let object_name = v.last().unwrap().clone();
        object_name.to_string()
    }

    pub(crate) fn get_file_target(&self) -> &str {
        self.file_target.get_value()
    }

    pub(crate) fn set_file_target<S: Into<String>>(&mut self, value: S) -> &mut Self {
        self.file_target.set_value(value);
        self
    }

    pub(crate) fn get_relationship_list(&self) -> &Vec<RawRelationship> {
        &self.relationship_list
    }

    pub(crate) fn _get_relationship_list_mut(&mut self) -> &mut Vec<RawRelationship> {
        &mut self.relationship_list
    }

    pub(crate) fn get_drawing_raw_file(&self) -> Option<&RawFile> {
        for relationship in self.get_relationship_list() {
            if relationship.get_type()
                == "http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing"
            {
                return Some(relationship.get_raw_file());
            }
        }
        None
    }

    pub(crate) fn get_vml_drawing_raw_file(&self) -> Option<&RawFile> {
        for relationship in self.get_relationship_list() {
            if relationship.get_type()
                == "http://schemas.openxmlformats.org/officeDocument/2006/relationships/vmlDrawing"
            {
                return Some(relationship.get_raw_file());
            }
        }
        None
    }

    pub(crate) fn get_relationship_by_rid<S: Into<String>>(&self, r_id: S) -> &RawRelationship {
        let r_id_str = r_id.into();
        for relationship in self.get_relationship_list() {
            if relationship.get_id() == r_id_str.as_str() {
                return relationship;
            }
        }
        panic!("not found relationship as {}.", &r_id_str);
    }

    pub(crate) fn add_relationship_list(&mut self, value: RawRelationship) -> &mut Self {
        self.relationship_list.push(value);
        self
    }

    pub(crate) fn set_attributes<R: io::Read + io::Seek>(
        &mut self,
        arv: &mut zip::read::ZipArchive<R>,
        base_path: &str,
        target: &str,
    ) {
        let data = {
            let path_str = normalize_path_to_str(&format!("{}/{}", base_path, target));
            let file_path = match arv.by_name(&path_str) {
                Ok(v) => v,
                Err(_) => {
                    return;
                }
            };
            self.set_file_target(path_str);
            let mut r = io::BufReader::new(file_path);
            let mut buf = Vec::new();
            r.read_to_end(&mut buf).unwrap();
            std::io::Cursor::new(buf)
        };
        let mut reader = Reader::from_reader(data);
        reader.trim_text(true);
        let mut buf = Vec::new();

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Empty(ref e)) => match e.name() {
                    b"Relationship" => {
                        let mut obj = RawRelationship::default();
                        obj.set_attributes(&mut reader, e, arv, base_path);
                        self.add_relationship_list(obj);
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
            buf.clear();
        }
    }

    pub(crate) fn write_to<W: io::Seek + io::Write>(
        &self,
        writer_mng: &mut WriterManager<W>,
        ather_target: Option<&str>,
    ) {
        if self.get_relationship_list().len() == 0 {
            return;
        }

        let mut writer = Writer::new(io::Cursor::new(Vec::new()));
        // XML header
        let _ = writer.write_event(Event::Decl(BytesDecl::new(
            b"1.0",
            Some(b"UTF-8"),
            Some(b"yes"),
        )));
        write_new_line(&mut writer);

        // relationships
        write_start_tag(
            &mut writer,
            "Relationships",
            vec![(
                "xmlns",
                "http://schemas.openxmlformats.org/package/2006/relationships",
            )],
            false,
        );

        for relationship in self.get_relationship_list() {
            relationship.write_to(&mut writer);
        }

        write_end_tag(&mut writer, "Relationships");

        let target = match ather_target {
            Some(v) => v,
            None => self.get_file_target(),
        };
        writer_mng.add_writer(target, writer);

        for relationship in self.get_relationship_list() {
            relationship.write_to_bin(writer_mng);
        }
    }
}