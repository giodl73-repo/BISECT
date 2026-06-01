use std::path::PathBuf;

use shapefile::dbase::{FieldValue, Record, TableWriterBuilder};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("fixtures")
        .join("import-label")
}

fn write_fixture(path: PathBuf, include_district: bool) {
    let mut table_builder =
        TableWriterBuilder::new().add_character_field("GEOID".try_into().unwrap(), 16);
    if include_district {
        table_builder = table_builder.add_numeric_field("DISTRICT".try_into().unwrap(), 8, 0);
    }

    let mut writer = shapefile::Writer::from_path(&path, table_builder)
        .unwrap_or_else(|e| panic!("cannot create '{}': {e}", path.display()));
    let records = [
        ("50001000100", 1.0, shapefile::Point::new(0.0, 0.0)),
        ("50001000200", 2.0, shapefile::Point::new(1.0, 1.0)),
    ];

    for (geoid, district, point) in records {
        let mut record = Record::default();
        record.insert(
            "GEOID".to_string(),
            FieldValue::Character(Some(geoid.to_string())),
        );
        if include_district {
            record.insert("DISTRICT".to_string(), FieldValue::Numeric(Some(district)));
        }
        writer
            .write_shape_and_record(&point, &record)
            .unwrap_or_else(|e| panic!("cannot write '{}': {e}", path.display()));
    }
}

fn main() {
    let root = fixture_root();
    let positive = root.join("positive");
    let negative = root.join("negative");
    std::fs::create_dir_all(&positive).expect("create positive fixture dir");
    std::fs::create_dir_all(&negative).expect("create negative fixture dir");

    write_fixture(positive.join("vermont_two_tracts.shp"), true);
    write_fixture(negative.join("shapefile_missing_district.shp"), false);

    println!("wrote shapefile fixtures under {}", root.display());
}
