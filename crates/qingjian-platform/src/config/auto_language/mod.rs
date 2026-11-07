//! 原地增删应用语言规则；每次先读磁盘，保留其他配置与注释。

use std::path::Path;

use qingjian_auto_language::{AutoLanguageConfig, LanguageRule};
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table};

use crate::Config;

impl Config {
    pub fn set_auto_language_rule(path: &Path, rule: &LanguageRule) -> Result<(), String> {
        let mut document = read_document(path)?;
        let mut rules = read_rules(&document)?;
        if let Some(previous) = rules
            .iter_mut()
            .find(|old| old.app_id.eq_ignore_ascii_case(&rule.app_id))
        {
            *previous = rule.clone();
        } else {
            rules.push(rule.clone());
        }
        write_rules(path, &mut document, &rules)
    }

    pub fn remove_auto_language_rule(path: &Path, app_id: &str) -> Result<(), String> {
        let mut document = read_document(path)?;
        let mut rules = read_rules(&document)?;
        rules.retain(|rule| !rule.app_id.eq_ignore_ascii_case(app_id));
        write_rules(path, &mut document, &rules)
    }
}

fn read_document(path: &Path) -> Result<DocumentMut, String> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => super::TEMPLATE.to_owned(),
        Err(error) => return Err(error.to_string()),
    };
    source
        .parse()
        .map_err(|error: toml_edit::TomlError| error.to_string())
}

fn read_rules(document: &DocumentMut) -> Result<Vec<LanguageRule>, String> {
    toml::from_str::<Config>(&document.to_string())
        .map(|config| config.auto_language.rules)
        .map_err(|error| error.to_string())
}

fn write_rules(
    path: &Path,
    document: &mut DocumentMut,
    rules: &[LanguageRule],
) -> Result<(), String> {
    AutoLanguageConfig {
        rules: rules.to_vec(),
        ..AutoLanguageConfig::default()
    }
    .validate()?;
    let section = super::section_table(document, "auto_language");
    let old = section.get("rules").and_then(Item::as_array_of_tables);
    let mut tables = ArrayOfTables::new();
    for rule in rules {
        let mut table = old
            .and_then(|tables| {
                tables.iter().find(|table| {
                    table
                        .get("app_id")
                        .and_then(Item::as_str)
                        .is_some_and(|id| id.eq_ignore_ascii_case(&rule.app_id))
                })
            })
            .cloned()
            .unwrap_or_else(Table::new);
        table["app_id"] = toml_edit::value(&rule.app_id);
        table["display_name"] = toml_edit::value(&rule.display_name);
        table["language"] = toml_edit::value(
            rule.language
                .map_or("default", qingjian_auto_language::InputLanguage::key),
        );
        tables.push(table);
    }
    section["rules"] = Item::ArrayOfTables(tables);
    super::write_file(path, &document.to_string()).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
