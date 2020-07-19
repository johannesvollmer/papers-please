use crate::Paper;
use std::path::Path;
use std::collections::HashMap;


fn parse_declarations(string: String) -> Result<HashMap<String, Vec<String>>, String> {
    let string = string.trim();
    if string.is_empty() { return Ok(HashMap::new()); }

    let mut values = HashMap::new();
    let mut current_identifier = None;

    let mut tokens = string.split('`');
    while let Some(identifier) = tokens.next() {
        let identifier = identifier.trim().to_owned();

        // if no identifier, continue with previous identifier
        if identifier.is_empty() {
            if let Some(value) = tokens.next() {
                let last_identifier = current_identifier.clone()
                    .ok_or(format!("expected identifier, but file begins with value `{}`", value));

                dbg!(&last_identifier);

                values.entry(last_identifier?)
                    .or_insert_with(|| Vec::new())
                    .push(value.to_owned());
            }
            // else: end of file (after empty identifier)
        }

        // start new identifier
        else {
            current_identifier = Some(identifier.clone());

            let value = tokens.next()
                .ok_or(format!("expected at least one value for identifier '{}'", identifier))?.to_owned();

            if values.insert(identifier.clone(), vec![ value ]).is_some() {
                return Err(format!("'{}' is defined multiple times", identifier))
            }
        }
    }

    dbg!(&values);
    Ok(values)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test(){
        let source = r#"
            x
            `y`
            `z`
        "#;

        let values = super::parse_declarations(source.to_owned()).unwrap();
    }

}