use crate::lib::model::transform_config::CaseType;

pub fn convert_case(str: &str, case_type: &CaseType) -> String {
    let mut result = str.to_owned();

    if case_type == &CaseType::UpperCamelCase {
        result[0..=0].make_ascii_uppercase();
    }

    for (i, char) in str.chars().enumerate() {
        match char {
            'A'..='Z' => {
                match case_type {
                    CaseType::SnakeCase => {
                        if i != 0 {
                            result.insert(i, '_');
                            result[i + 1..=i + 1].make_ascii_lowercase();
                        } else {
                            result[i..=i].make_ascii_lowercase();
                        }
                    },
                    _ => {}
                }
            },
            '_' | '-' => {
                match case_type {
                    CaseType::SnakeCase => {
                        result = result.replace('-', "_");
                        return result;
                    }
                    CaseType::CamelCase | CaseType::UpperCamelCase  => {
                        if i != 0 {
                            // Absolutely ugly way of ignoring the first char of the string (in case it is a '_').
                            let index = result[1..].find(char).unwrap() + 1;
                            result.remove(index);
                            result[index..=index].make_ascii_uppercase();
                        }
                    },
                }
            },
            _ => {},
        }

    }

    result
}

#[cfg(test)]
mod tests {
    use crate::lib::case::{CaseType, convert_case};

    #[test]
    fn camel_to_snake() {
        let str = "hoLa";
        let expected_result = String::from("ho_la");
        let result = convert_case(str, &CaseType::SnakeCase);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn camel_to_snake_first_uppercase() {
        let str = "HoLa";
        let expected_result = String::from("ho_la");
        let result = convert_case(str, &CaseType::SnakeCase);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn snake_to_camel() {
        let str = "ho_la";
        let expected_result = String::from("hoLa");
        let result = convert_case(str, &CaseType::CamelCase);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn snake_to_camel_first_underscore() {
        let str = "_ho_la";
        let expected_result = String::from("_hoLa");
        let result = convert_case(str, &CaseType::CamelCase);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn kebab_to_camel() {
        let str = "ho-la";
        let expected_result = String::from("hoLa");
        let result = convert_case(str, &CaseType::CamelCase);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn kebab_to_snake() {
        let str = "ho-la";
        let expected_result = String::from("ho_la");
        let result = convert_case(str, &CaseType::SnakeCase);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn multiple_snake_to_camel() {
        let str = "ho_la_eh";
        let expected_result = String::from("hoLaEh");
        let result = convert_case(str, &CaseType::CamelCase);

        assert_eq!(result, expected_result);
    }
}