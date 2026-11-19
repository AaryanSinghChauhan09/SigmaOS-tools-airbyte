#![forbid(unsafe_code)]
#![allow(non_snake_case)]
//! Airbyte ELT Connector Engine for SigmaOS
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct AirbyteConnector {
    pub stream_name: String,
    pub schema: Vec<SchemaField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub values: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyncReport {
    pub records_extracted: usize,
    pub records_transformed: usize,
    pub records_loaded: usize,
    pub validation_errors: usize,
}

impl AirbyteConnector {
    pub fn new(stream_name: &str, schema: Vec<SchemaField>) -> Self {
        Self {
            stream_name: stream_name.to_string(),
            schema,
        }
    }

    pub fn extract(&self, raw_data: Vec<HashMap<String, String>>) -> Vec<Record> {
        raw_data
            .into_iter()
            .map(|values| Record { values })
            .collect()
    }

    pub fn transform(&self, records: Vec<Record>) -> (Vec<Record>, usize) {
        let mut valid_records = Vec::new();
        let mut errors = 0;

        for record in records {
            let mut valid = true;
            let mut transformed_values = HashMap::new();

            for field in &self.schema {
                if let Some(val) = record.values.get(&field.name) {
                    let coerced_valid = match field.field_type {
                        FieldType::String => true,
                        FieldType::Integer => val.trim().parse::<i64>().is_ok(),
                        FieldType::Float => val.trim().parse::<f64>().is_ok(),
                        FieldType::Boolean => {
                            let lower = val.to_lowercase();
                            lower == "true" || lower == "false" || lower == "1" || lower == "0"
                        }
                    };
                    if coerced_valid {
                        transformed_values.insert(field.name.clone(), val.trim().to_string());
                    } else {
                        valid = false;
                        break;
                    }
                } else if field.required {
                    valid = false;
                    break;
                }
            }

            if valid {
                valid_records.push(Record {
                    values: transformed_values,
                });
            } else {
                errors += 1;
            }
        }

        (valid_records, errors)
    }

    pub fn load(&self, destination_sink: &mut Vec<Record>, records: Vec<Record>) -> usize {
        let count = records.len();
        destination_sink.extend(records);
        count
    }

    pub fn sync(
        &self,
        raw_data: Vec<HashMap<String, String>>,
        destination_sink: &mut Vec<Record>,
    ) -> SyncReport {
        let extracted = self.extract(raw_data);
        let ext_count = extracted.len();
        let (transformed, err_count) = self.transform(extracted);
        let trans_count = transformed.len();
        let loaded = self.load(destination_sink, transformed);

        SyncReport {
            records_extracted: ext_count,
            records_transformed: trans_count,
            records_loaded: loaded,
            validation_errors: err_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airbyte_elt_pipeline_sync_aaa() {
        // Arrange
        let schema = vec![
            SchemaField {
                name: "id".to_string(),
                field_type: FieldType::Integer,
                required: true,
            },
            SchemaField {
                name: "username".to_string(),
                field_type: FieldType::String,
                required: true,
            },
            SchemaField {
                name: "active".to_string(),
                field_type: FieldType::Boolean,
                required: false,
            },
        ];
        let connector = AirbyteConnector::new("users_stream", schema);

        let mut row1 = HashMap::new();
        row1.insert("id".to_string(), "101".to_string());
        row1.insert("username".to_string(), "aaryan".to_string());
        row1.insert("active".to_string(), "true".to_string());

        let mut row2_invalid = HashMap::new();
        row2_invalid.insert("id".to_string(), "not_an_int".to_string());
        row2_invalid.insert("username".to_string(), "bob".to_string());

        let mut sink = Vec::new();

        // Act
        let report = connector.sync(vec![row1, row2_invalid], &mut sink);

        // Assert
        assert_eq!(report.records_extracted, 2);
        assert_eq!(report.records_transformed, 1);
        assert_eq!(report.records_loaded, 1);
        assert_eq!(report.validation_errors, 1);
        assert_eq!(sink[0].values.get("username").unwrap(), "aaryan");
    }
}

#[cfg(test)]
mod airbyte_extra_tests {
    use super::*;

    #[test]
    fn test_airbyte_empty_stream_extraction_aaa() {
        // Arrange
        let connector = AirbyteConnector::new("empty_stream", Vec::new());
        // Act
        let records = connector.extract(Vec::new());
        // Assert
        assert!(records.is_empty());
    }

    #[test]
    fn test_airbyte_stream_name_getter_aaa() {
        // Arrange
        let connector = AirbyteConnector::new("metrics_stream", Vec::new());
        // Act & Assert
        assert_eq!(connector.stream_name, "metrics_stream");
    }

    #[test]
    fn test_airbyte_sync_report_success_rate_aaa() {
        // Arrange
        let report = SyncReport {
            records_extracted: 10,
            records_transformed: 8,
            records_loaded: 8,
            validation_errors: 2,
        };
        // Act & Assert
        assert_eq!(report.records_extracted, 10);
        assert_eq!(report.validation_errors, 2);
    }
}
