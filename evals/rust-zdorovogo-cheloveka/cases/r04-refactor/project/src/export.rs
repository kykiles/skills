use crate::Entry;

/// Выгрузка событий в CSV.
pub fn format_csv(entries: &[Entry]) -> String {
    let mut out = String::from("source,level,message,count\n");
    for entry in entries {
        out.push_str(&format!(
            "{},{},{},{}\n",
            entry.source, entry.level, entry.message, entry.count
        ));
    }
    out
}
