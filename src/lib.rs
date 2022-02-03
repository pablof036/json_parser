use std::fs;

pub fn run(filename: String) -> anyhow::Result<()> {
    let file = fs::read_to_string(filename)?;


    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::run;

    #[test]
    fn runs_on_valid_path() -> anyhow::Result<()>{
        run(String::from("test.json"))
    }

    #[test]
    #[should_panic]
    fn fails_on_valid_path() {
        run(String::from("a")).unwrap()
    }
}