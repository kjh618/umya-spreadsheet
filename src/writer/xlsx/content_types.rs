use quick_xml::events::{Event, BytesDecl};
use quick_xml::Writer;
use std::io::Cursor;
use tempdir::TempDir;

use super::super::structs::spreadsheet::Spreadsheet;
use super::driver::*;
use super::XlsxError;

pub(crate) fn write(spreadsheet: &Spreadsheet, dir: &TempDir, file_name: &str) -> Result<(), XlsxError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    // XML header
    let _ = writer.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), Some(b"yes"))));
    write_new_line(&mut writer);

    // Types
    write_start_tag(&mut writer, "Types", vec![
        ("xmlns", "http://schemas.openxmlformats.org/package/2006/content-types"),
    ], false);

    // Default bin
    //write_start_tag(&mut writer, "Default", vec![
    //    ("Extension", "bin"),
    //    ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.printerSettings"),
    //], true);

    // Default rels
    write_start_tag(&mut writer, "Default", vec![
        ("Extension", "rels"),
        ("ContentType", "application/vnd.openxmlformats-package.relationships+xml"),
    ], true);

    // Default xml
    write_start_tag(&mut writer, "Default", vec![
        ("Extension", "xml"),
        ("ContentType", "application/xml"),
    ], true);
    
    // Override workbook
    write_start_tag(&mut writer, "Override", vec![
        ("PartName", "/xl/workbook.xml"),
        ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"),
    ], true);

    for i in 0..spreadsheet.get_sheet_count() {
        // Override sheet
        write_start_tag(&mut writer, "Override", vec![
            ("PartName", format!("/xl/worksheets/sheet{}.xml", (i+1).to_string().as_str()).as_str()),
            ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"),
        ], true);
    }

    // Override theme
    write_start_tag(&mut writer, "Override", vec![
        ("PartName", "/xl/theme/theme1.xml"),
        ("ContentType", "application/vnd.openxmlformats-officedocument.theme+xml"),
    ], true);

    // Override styles
    write_start_tag(&mut writer, "Override", vec![
        ("PartName", "/xl/styles.xml"),
        ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"),
    ], true);

    // Override sharedStrings
    write_start_tag(&mut writer, "Override", vec![
        ("PartName", "/xl/sharedStrings.xml"),
        ("ContentType", "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"),
    ], true);

    let mut drawing_count = 1;
    let mut chart_count = 1;
    for worksheet in spreadsheet.get_sheet_collection() {
        //for _ in worksheet.get_drawing_collection() {
        //    // Override drawing
        //    write_start_tag(&mut writer, "Override", vec![
        //        ("PartName", format!("/xl/drawings/drawing{}.xml", drawing_count.to_string().as_str()).as_str()),
        //        ("ContentType", "application/vnd.openxmlformats-officedocument.drawing+xml"),
        //    ], true);
        //    drawing_count += 1;
        //}
        if worksheet.get_chart_collection().len() > 0 {
            // Override drawing
            write_start_tag(&mut writer, "Override", vec![
                ("PartName", format!("/xl/drawings/drawing{}.xml", drawing_count.to_string().as_str()).as_str()),
                ("ContentType", "application/vnd.openxmlformats-officedocument.drawing+xml"),
            ], true);
            drawing_count += 1;
        }
        for _ in worksheet.get_chart_collection() {
            // Override chart
            write_start_tag(&mut writer, "Override", vec![
                ("PartName", format!("/xl/charts/chart{}.xml", chart_count.to_string().as_str()).as_str()),
                ("ContentType", "application/vnd.openxmlformats-officedocument.drawingml.chart+xml"),
            ], true);
            chart_count += 1;
        }
    }

    // Override docProps/core
    write_start_tag(&mut writer, "Override", vec![
        ("PartName", "/docProps/core.xml"),
        ("ContentType", "application/vnd.openxmlformats-package.core-properties+xml"),
    ], true);

    // Override docProps/app
    write_start_tag(&mut writer, "Override", vec![
        ("PartName", "/docProps/app.xml"),
        ("ContentType", "application/vnd.openxmlformats-officedocument.extended-properties+xml"),
    ], true);

    write_end_tag(&mut writer, "Types");
    let _ = make_file_from_writer(format!("{}",file_name).as_str(), dir, writer, None).unwrap();
    Ok(())
}