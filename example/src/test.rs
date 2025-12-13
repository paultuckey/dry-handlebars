
#[cfg(test)]
mod tests {

    #[test]
    fn basic_usage() {
        mod template {
            dry_handlebars::str!("test", r#"<p>{{firstname}} {{lastname}}</p>"#);
        }
        assert_eq!(
            template::test(" King", "Tubby ").render(),
            "<p> King Tubby </p>"
        );
    }

    #[test]
    fn path_expressions() {
        mod template {
            dry_handlebars::str!("test", r#"{{person.firstname}} {{person.lastname}}"#);
        }
        assert_eq!(
            template::test("King", "Tubby").render(),
            "King Tubby"
        );
    }

}