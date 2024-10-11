use regex::Regex;

pub fn file_path_cleaner(title: &str) -> String {
    let mut cleaned_title = title.replace('\n', "")
        .replace('\r', "")
        .trim()
        .to_string();
    let re = Regex::new(r#"[/\\:*?"<>|]"#).unwrap();
    cleaned_title = re.replace_all(&cleaned_title, "_").to_string();
    cleaned_title = cleaned_title.replace(' ', "_");
    cleaned_title
}