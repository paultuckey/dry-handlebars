pub use dry_handlebars_macros::dry_handlebars_directory as directory;
pub use dry_handlebars_macros::dry_handlebars_file as file;
pub use dry_handlebars_macros::dry_handlebars_str as str;

#[cfg(test)]
mod tests {

    #[test]
    fn basic_usage() {
        mod template {
            crate::str!("test", r#"<p>{{firstname}} {{lastname}}</p>"#);
        }
        assert_eq!(
            template::test("King", "Tubby").render(),
            "<p>King Tubby</p>"
        );
    }

    struct Person {
        firstname: String,
        lastname: String,
    }

    #[test]
    fn path_expressions() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"{{person.firstname}} {{person.lastname}}"#,
                ("person", super::Person)
            );
        }
        let person = Person {
            firstname: "King".to_string(),
            lastname: "Tubby".to_string(),
        };
        assert_eq!(template::test(person).render(), "King Tubby");
    }

    struct Author {
        first_name: String,
        last_name: String,
    }

    #[test]
    fn if_bool_helper() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"<div>{{#if has_author}}<h1>{{first_name}} {{last_name}}</h1>{{/if}}</div>"#
            );
        }

        assert_eq!(
            template::test(true, "King", "Tubby").render().trim(),
            //language=html
            "<div><h1>King Tubby</h1></div>"
        );
        assert_eq!(
            template::test(false, "King", "Tubby").render().trim(),
            //language=html
            "<div></div>"
        );
    }

    #[test]
    fn if_option_helper() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"<div>{{#if author}}<h1>{{first_name}} {{last_name}}</h1>{{/if}}</div>"#,
                ("author", Option<super::Author>)
            );
        }
        let author = Author {
            first_name: "King".to_string(),
            last_name: "Tubby".to_string(),
        };
        assert_eq!(
            template::test(Some(author)).render().trim(),
            //language=html
            "<div><h1>King Tubby</h1></div>"
        );
        assert_eq!(
            template::test(None).render().trim(),
            //language=html
            "<div></div>"
        );
    }

    #[test]
    fn if_else_option_helper() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"<div>{{#if author}}<h1>{{first_name}}</h1>{{else}}<h1>Unknown</h1>{{/if}}</div>"#,
                ("author", Option<super::Author>)
            );
        }
        let author = Author {
            first_name: "King".to_string(),
            last_name: "Tubby".to_string(),
        };
        assert_eq!(
            template::test(Some(author)).render().trim(),
            //language=html
            r#"<div><h1>King</h1></div>"#
        );
        assert_eq!(
            template::test(None).render().trim(),
            //language=html
            r#"<div><h1>Unknown</h1></div>"#
        );
    }

    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///

    #[test]
    fn it_works() {
        mod template {
            crate::str!("test", "Hello {{{name}}}!");
        }
        assert_eq!(template::test("King").render(), "Hello King!");
    }

    #[test]
    fn test_escaped() {
        mod template {
            crate::str!(
                "test",
                "{{{{skip}}}}wang doodle {{{{/dandy}}}}{{{{/skip}}}}"
            );
        }
        assert_eq!(template::test().render(), "wang doodle {{{{/dandy}}}}");
    }

    #[test]
    fn test_format_number() {
        mod template {
            crate::str!("test", "Price: ${{format \"{:.2}\" price}}");
        }
        assert_eq!(template::test(12.2345f64).render(), "Price: $12.23");
    }
}
