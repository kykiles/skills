use crate::Entry;

/// Выгрузка событий в CSV.
pub fn format_csv(entries: &[Entry]) -> String {
    let mut out = String::from("source,level,message,count\n");
    for entry in entries {
        // TODO(маша): source тоже может содержать запятую
        out.push_str(&format!(
            "{},{},{},{}\n",
            entry.source,
            entry.level,
            quote(&entry.message),
            entry.count
        ));
    }
    out
}

fn quote(field: &str) -> String {
    if field.contains([',', '"', '\n']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
