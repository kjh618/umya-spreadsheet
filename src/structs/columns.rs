// fills
use structs::Cells;
use structs::Column;
use structs::ColumnSort;
use structs::MergeCells;
use structs::Stylesheet;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use reader::driver::*;
use std::io::Cursor;
use writer::driver::*;

#[derive(Clone, Default, Debug)]
pub(crate) struct Columns {
    column: Vec<Column>,
}
impl Columns {
    pub(crate) fn get_column_collection(&self) -> &Vec<Column> {
        &self.column
    }

    pub(crate) fn get_column_collection_mut(&mut self) -> &mut Vec<Column> {
        &mut self.column
    }

    pub(crate) fn get_column(&self, value: &u32) -> Option<&Column> {
        for column in &self.column {
            if value == column.get_col_num() {
                return Some(column);
            }
        }
        None
    }

    pub(crate) fn get_column_mut(&mut self, value: &u32) -> &mut Column {
        match self.get_column(value) {
            Some(_) => {}
            None => {
                let mut obj = Column::default();
                obj.set_col_num(value.clone());
                self.set_column(obj);
            }
        }
        for column in self.get_column_collection_mut() {
            if value == column.get_col_num() {
                return column;
            }
        }
        panic!("Column not found.");
    }

    pub(crate) fn set_column(&mut self, value: Column) -> &mut Self {
        self.column.push(value);
        self
    }

    pub(crate) fn calculation_auto_width(
        &mut self,
        cells: &Cells,
        merge_cells: &MergeCells,
    ) -> &mut Self {
        for column in self.get_column_collection_mut() {
            let has_horizontal = merge_cells.has_horizontal(column.get_col_num());
            if has_horizontal {
                continue;
            }
            column.calculation_auto_width(cells);
        }
        self
    }

    pub(crate) fn adjustment_insert_coordinate(
        &mut self,
        root_col_num: &u32,
        offset_col_num: &u32,
    ) {
        for column_dimension in self.get_column_collection_mut() {
            column_dimension.adjustment_insert_coordinate(root_col_num, offset_col_num);
        }
    }

    pub(crate) fn adjustment_remove_coordinate(
        &mut self,
        root_col_num: &u32,
        offset_col_num: &u32,
    ) {
        self.get_column_collection_mut().retain(|x| {
            !(x.get_col_num() > root_col_num && x.get_col_num() < &(root_col_num + offset_col_num))
        });
        for column_dimension in self.get_column_collection_mut() {
            column_dimension.adjustment_remove_coordinate(root_col_num, offset_col_num);
        }
    }

    pub(crate) fn set_attributes<R: std::io::BufRead>(
        &mut self,
        reader: &mut Reader<R>,
        _e: &BytesStart,
        stylesheet: &Stylesheet,
    ) {
        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Empty(ref e)) => match e.name() {
                    b"col" => {
                        let mut obj = Column::default();
                        obj.set_attributes(reader, e, stylesheet);
                        let min = get_attribute(e, b"min").unwrap().parse::<u32>().unwrap();
                        let max = get_attribute(e, b"max").unwrap().parse::<u32>().unwrap();
                        for i in min..=max {
                            obj.set_col_num(i);
                            self.set_column(obj.clone());
                        }
                    }
                    _ => (),
                },
                Ok(Event::End(ref e)) => match e.name() {
                    b"cols" => return,
                    _ => (),
                },
                Ok(Event::Eof) => panic!("Error not find {} end element", "cols"),
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
            buf.clear();
        }
    }

    pub(crate) fn write_to(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        stylesheet: &mut Stylesheet,
    ) {
        if self.column.len() > 0 {
            let mut column_index: Vec<ColumnSort> = Vec::new();
            for column in self.get_column_collection() {
                let mut obj = ColumnSort::default();
                obj.col_num = column.get_col_num().clone();
                obj.hash_code = column.get_hash_code();
                column_index.push(obj);
            }
            column_index.sort();

            // cols
            write_start_tag(writer, "cols", vec![], false);

            // col
            let mut hash_code = column_index[0].hash_code.clone();
            let mut min = column_index[0].col_num.clone();
            let mut max = column_index[0].col_num.clone();
            for index in &column_index {
                if max + 1 >= index.col_num && hash_code == index.hash_code {
                    max = index.col_num.clone();
                } else {
                    for column in self.get_column_collection() {
                        if &min == column.get_col_num() {
                            self.write_to_column(writer, &min, &max, column, stylesheet);
                        }
                    }
                    hash_code = index.hash_code.clone();
                    min = index.col_num.clone();
                    max = index.col_num.clone();
                }
            }
            for column in self.get_column_collection() {
                if &min == column.get_col_num() {
                    self.write_to_column(writer, &min, &max, column, stylesheet);
                }
            }

            write_end_tag(writer, "cols");
        }
    }

    pub(crate) fn write_to_column(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        min: &u32,
        max: &u32,
        column: &Column,
        stylesheet: &mut Stylesheet,
    ) {
        // col
        let mut attributes: Vec<(&str, &str)> = Vec::new();
        let min_str = min.to_string();
        let max_str = max.to_string();
        attributes.push(("min", min_str.as_str()));
        attributes.push(("max", max_str.as_str()));
        attributes.push(("width", column.width.get_value_string()));
        if column.hidden.get_value() == &true {
            attributes.push(("hidden", column.hidden.get_value_string()));
        }
        if column.best_fit.get_value() == &true {
            attributes.push(("bestFit", column.best_fit.get_value_string()));
        }
        attributes.push(("customWidth", "1"));
        let xf_index_str: String;
        let xf_index = stylesheet.set_style(column.get_style());
        if xf_index > 0 {
            xf_index_str = xf_index.to_string();
            attributes.push(("style", &xf_index_str));
        }
        write_start_tag(writer, "col", attributes, true);
    }
}